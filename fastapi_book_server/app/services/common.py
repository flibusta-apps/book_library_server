from typing import Optional, Generic, TypeVar, Union
from itertools import permutations
import json

from fastapi_pagination.api import resolve_params
from fastapi_pagination.bases import AbstractParams, RawParams
from app.utils.pagination import Page, CustomPage

from ormar import Model, QuerySet
from sqlalchemy import text, func, select, or_, Table, Column
from sqlalchemy.orm import Session
from databases import Database


def join_fields(fields):
    result = fields[0]

    for el in fields[1:]:
        result += text("' '") + el

    return result


T = TypeVar('T', bound=Model)


class TRGMSearchService(Generic[T]):
    MODEL_CLASS: Optional[T] = None
    FIELDS: Optional[list[Column]] = None
    SELECT_RELATED: Optional[Union[list[str], str]] = None
    PREFETCH_RELATED: Optional[Union[list[str], str]] = None

    @classmethod
    def get_params(cls) -> AbstractParams:
        return resolve_params()

    @classmethod
    def get_raw_params(cls) -> RawParams:
        return resolve_params().to_raw_params()

    @classmethod
    @property
    def model(cls) -> T:
        assert cls.MODEL_CLASS is not None, f"MODEL in {cls.__name__} don't set!"
        return cls.MODEL_CLASS

    @classmethod
    @property
    def table(cls) -> Table:
        return cls.model.Meta.table

    @classmethod
    @property
    def database(cls) -> Database:
        return cls.model.Meta.database

    @classmethod
    @property
    def fields_combinations(cls):
        assert cls.FIELDS is not None, f"FIELDS in {cls.__name__} don't set!"
        assert len(cls.FIELDS) != 0, f"FIELDS in {cls.__name__} must be not empty!"

        return permutations(cls.FIELDS, len(cls.FIELDS))

    @classmethod
    def get_similarity_subquery(cls, query: str):
        combs = cls.fields_combinations

        return func.greatest(
            *[func.similarity(join_fields(comb), f"{query}::text") for comb in combs]
        ).label("sml")

    @classmethod
    def get_similarity_filter_subquery(cls, query: str):
        return or_(
            *[join_fields(comb) % f"{query}::text" for comb in cls.fields_combinations]
        )

    @classmethod
    async def get_objects(cls, query_data: str) -> tuple[int, list[T]]:
        similarity = cls.get_similarity_subquery(query_data)
        similarity_filter = cls.get_similarity_filter_subquery(query_data)

        params = cls.get_raw_params()
        
        session = Session(cls.database.connection())

        q1 = session.query(
            cls.table.c.id, similarity
        ).order_by(
            text('sml DESC')
        ).filter(
            cls.table.c.is_deleted == False,
            similarity_filter
        ).cte('objs')

        sq = session.query(q1.c.id).limit(params.limit).offset(params.offset).subquery()

        q2 = session.query(
            func.json_build_object(
                text("'total'"), func.count(q1.c.id),
                text("'items'"), select(func.array_to_json(func.array_agg(sq.c.id)))
            )
        ).cte()

        print(str(q2))

        row = await cls.database.fetch_one(q2)

        if row is None:
            raise ValueError('Something is wrong!')

        result = json.loads(row['json_build_object_1'])

        queryset: QuerySet[T] = cls.model.objects

        if cls.PREFETCH_RELATED is not None:
            queryset = queryset.prefetch_related(cls.PREFETCH_RELATED)

        if cls.SELECT_RELATED:
            queryset = queryset.select_related(cls.SELECT_RELATED)

        return result['total'], await queryset.filter(id__in=result['items']).all()
        

    @classmethod
    async def get(cls, query: str) -> Page[T]:
        params = cls.get_params()

        total, objects = await cls.get_objects(query)

        return CustomPage.create(
            items=objects,
            total=total,
            params=params
        )
