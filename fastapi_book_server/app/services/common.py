from typing import Optional, Generic, TypeVar, Union, cast
from itertools import permutations

from fastapi_pagination.api import resolve_params
from fastapi_pagination.bases import AbstractParams, RawParams
from app.utils.pagination import Page, CustomPage

from ormar import Model, QuerySet
from sqlalchemy import text, func, select, desc, Table, Column
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

        if len(cls.FIELDS) == 1:
            return cls.FIELDS

        combinations = []

        for i in range(1, len(cls.FIELDS)):
            combinations += permutations(cls.FIELDS, i)

        return combinations

    @classmethod
    def get_similarity_subquery(cls, query: str):
        return func.greatest(
            *[func.similarity(join_fields(comb), f"{query}::text") for comb in cls.fields_combinations]
        ).label("sml")

    @classmethod
    def get_object_ids_query(cls, query: str):
        similarity = cls.get_similarity_subquery(query)
        params = cls.get_raw_params()

        return select(
            [cls.table.c.id],
        ).where(
            similarity > 0.5
        ).order_by(
            desc(similarity)
        ).limit(params.limit).offset(params.offset)

    @classmethod
    def get_objects_count_query(cls, query: str):
        similarity = cls.get_similarity_subquery(query)

        return select(
            func.count(cls.table.c.id)
        ).where(
            similarity > 0.5
        )

    @classmethod
    async def get_objects_count(cls, query: str) -> int:
        count_query = cls.get_objects_count_query(query)

        count_row = await cls.database.fetch_one(count_query)

        assert count_row is not None

        return cast(int, count_row.get("count_1"))

    @classmethod
    async def get_objects(cls, query: str) -> list[T]:
        ids_query = cls.get_object_ids_query(query)

        ids = await cls.database.fetch_all(ids_query)

        queryset: QuerySet[T] = cls.model.objects

        if cls.PREFETCH_RELATED is not None:
            queryset = queryset.prefetch_related(cls.PREFETCH_RELATED)

        if cls.SELECT_RELATED:
            queryset = queryset.select_related(cls.SELECT_RELATED)

        return await queryset.filter(id__in=[r.get("id") for r in ids]).all()

    @classmethod
    async def get(cls, query: str) -> Page[T]:
        params = cls.get_params()

        authors = await cls.get_objects(query)
        total = await cls.get_objects_count(query)

        return CustomPage.create(
            items=authors,
            total=total,
            params=params
        )
