from typing import Optional, Generic, TypeVar, Union
from databases import Database

from fastapi_pagination.api import resolve_params
from fastapi_pagination.bases import AbstractParams, RawParams
from app.utils.pagination import Page, CustomPage
import aioredis
import orjson

from ormar import Model, QuerySet
from sqlalchemy import Table


T = TypeVar('T', bound=Model)


class TRGMSearchService(Generic[T]):
    MODEL_CLASS: Optional[T] = None
    SELECT_RELATED: Optional[Union[list[str], str]] = None
    PREFETCH_RELATED: Optional[Union[list[str], str]] = None
    GET_OBJECT_IDS_QUERY: Optional[str] = None
    CACHE_TTL = 5 * 60

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
    def object_ids_query(cls) -> str:
        assert cls.GET_OBJECT_IDS_QUERY is not None, f"GET_OBJECT_IDS_QUERY in {cls.__name__} don't set!"
        return cls.GET_OBJECT_IDS_QUERY

    @classmethod
    async def _get_object_ids(cls, query_data: str) -> list[int]:
        row = await cls.database.fetch_one(cls.object_ids_query, {"query": query_data})

        if row is None:
            raise ValueError('Something is wrong!')

        return row['array']

    @classmethod
    def get_cache_key(cls, query_data: str) -> str:
        model_class_name = cls.model.__class__.__name__
        return f"{model_class_name}_{query_data}"

    @classmethod
    async def get_cached_ids(cls, query_data: str, redis: aioredis.Redis) -> Optional[list[int]]:
        try:
            key = cls.get_cache_key(query_data)
            data = await redis.get(key)

            if data is None:
                return data

            return orjson.loads(data)
        except aioredis.RedisError as e:
            print(e)
            return None

    @classmethod
    async def cache_object_ids(cls, query_data: str, object_ids: list[int], redis: aioredis.Redis):
        try:
            key = cls.get_cache_key(query_data)
            await redis.set(key, orjson.dumps(object_ids), ex=cls.CACHE_TTL)
        except aioredis.RedisError as e:
            print(e)

    @classmethod
    async def get_objects(cls, query_data: str, redis: aioredis.Redis) -> tuple[int, list[T]]:
        params = cls.get_raw_params()

        cached_object_ids = await cls.get_cached_ids(query_data, redis)

        if cached_object_ids is None:
            object_ids = await cls._get_object_ids(query_data)
            await cls.cache_object_ids(query_data, object_ids, redis)
        else:
            object_ids = cached_object_ids

        limited_object_ids = object_ids[params.offset:params.offset + params.limit]

        queryset: QuerySet[T] = cls.model.objects

        if cls.PREFETCH_RELATED is not None:
            queryset = queryset.prefetch_related(cls.PREFETCH_RELATED)

        if cls.SELECT_RELATED:
            queryset = queryset.select_related(cls.SELECT_RELATED)

        return len(object_ids), await queryset.filter(id__in=limited_object_ids).all()

    @classmethod
    async def get(cls, query: str, redis: aioredis.Redis) -> Page[T]:
        params = cls.get_params()

        total, objects = await cls.get_objects(query, redis)

        return CustomPage.create(
            items=objects,
            total=total,
            params=params
        )
