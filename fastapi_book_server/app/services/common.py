import abc
import asyncio
from concurrent.futures import ThreadPoolExecutor
from random import choice
from typing import Optional, Generic, TypeVar, TypedDict, Union

import aioredis
from databases import Database
from fastapi_pagination.api import resolve_params
from fastapi_pagination.bases import AbstractParams, RawParams
import meilisearch
from ormar import Model, QuerySet
from sqlalchemy import Table

from app.utils.pagination import Page, CustomPage
from core.config import env_config


MODEL = TypeVar("MODEL", bound=Model)
QUERY = TypeVar("QUERY", bound=TypedDict)


class BaseSearchService(Generic[MODEL, QUERY], abc.ABC):
    MODEL_CLASS: Optional[MODEL] = None
    SELECT_RELATED: Optional[Union[list[str], str]] = None
    PREFETCH_RELATED: Optional[Union[list[str], str]] = None
    CUSTOM_CACHE_PREFIX: Optional[str] = None
    CACHE_TTL = 6 * 60 * 60

    @classmethod
    def get_params(cls) -> AbstractParams:
        return resolve_params()

    @classmethod
    def get_raw_params(cls) -> RawParams:
        return resolve_params().to_raw_params()

    @classmethod
    @property
    def model(cls) -> MODEL:
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
    def cache_prefix(cls) -> str:
        return cls.CUSTOM_CACHE_PREFIX or cls.model.Meta.tablename

    @staticmethod
    def _get_query_hash(query: QUERY):
        return hash(frozenset(query.items()))

    @classmethod
    async def _get_object_ids(cls, query: QUERY) -> list[int]:
        ...

    @classmethod
    def get_cache_key(cls, query: QUERY) -> str:
        model_class_name = cls.cache_prefix
        query_hash = cls._get_query_hash(query)
        return f"{model_class_name}_{query_hash}"

    @classmethod
    async def get_cached_ids(
        cls,
        query: QUERY,
        redis: aioredis.Redis,
        params: RawParams,
    ) -> Optional[tuple[int, list[int]]]:
        try:
            key = cls.get_cache_key(query)
            active_key = f"{key}_active"

            if not await redis.exists(active_key):
                return None

            objects_count, objects = await asyncio.gather(
                redis.llen(key),
                redis.lrange(key, params.offset, params.offset + params.limit),
            )

            return objects_count, [int(item.decode()) for item in objects]
        except aioredis.RedisError as e:
            print(e)
            return None

    @classmethod
    async def cache_object_ids(
        cls,
        query: QUERY,
        object_ids: list[int],
        redis: aioredis.Redis,
    ) -> bool:
        try:
            key = cls.get_cache_key(query)
            active_key = f"{key}_active"

            p = redis.pipeline()

            await p.delete(key)
            await p.set(active_key, 1, ex=cls.CACHE_TTL)
            await p.rpush(key, *object_ids)

            await p.execute()

            return True
        except aioredis.RedisError as e:
            print(e)
            return False

    @classmethod
    async def get_object_ids(
        cls, query: QUERY, redis: aioredis.Redis
    ) -> tuple[int, list[int]]:
        params = cls.get_raw_params()

        if (
            cached_object_ids := await cls.get_cached_ids(query, redis, params)
        ) is not None:
            return cached_object_ids

        object_ids = await cls._get_object_ids(query)
        limited_object_ids = object_ids[params.offset : params.offset + params.limit]

        if len(object_ids) != 0:
            await cls.cache_object_ids(query, object_ids, redis)

        return len(object_ids), limited_object_ids

    @classmethod
    async def get_limited_objects(
        cls, query: QUERY, redis: aioredis.Redis
    ) -> tuple[int, list[MODEL]]:
        count, object_ids = await cls.get_object_ids(query, redis)

        queryset: QuerySet[MODEL] = cls.model.objects

        if cls.PREFETCH_RELATED is not None:
            queryset = queryset.prefetch_related(cls.PREFETCH_RELATED)

        if cls.SELECT_RELATED:
            queryset = queryset.select_related(cls.SELECT_RELATED)

        db_objects = await queryset.filter(id__in=object_ids).all()
        return count, sorted(db_objects, key=lambda o: object_ids.index(o.id))

    @classmethod
    async def get(cls, query: QUERY, redis: aioredis.Redis) -> Page[MODEL]:
        params = cls.get_params()

        total, objects = await cls.get_limited_objects(query, redis)

        return CustomPage.create(items=objects, total=total, params=params)


class SearchQuery(TypedDict):
    query: str
    allowed_langs: frozenset[str]


class TRGMSearchService(Generic[MODEL], BaseSearchService[MODEL, SearchQuery]):
    GET_OBJECT_IDS_QUERY: Optional[str] = None

    @classmethod
    @property
    def object_ids_query(cls) -> str:
        assert (
            cls.GET_OBJECT_IDS_QUERY is not None
        ), f"GET_OBJECT_IDS_QUERY in {cls.__name__} don't set!"
        return cls.GET_OBJECT_IDS_QUERY

    @classmethod
    async def _get_object_ids(cls, query: SearchQuery) -> list[int]:
        row = await cls.database.fetch_one(
            cls.object_ids_query,
            {"query": query["query"], "langs": query["allowed_langs"]},
        )

        if row is None:
            raise ValueError("Something is wrong!")

        return row["array"]


class MeiliSearchService(Generic[MODEL], BaseSearchService[MODEL, SearchQuery]):
    MS_INDEX_NAME: Optional[str] = None
    MS_INDEX_LANG_KEY: Optional[str] = None

    _executor = ThreadPoolExecutor(2)

    @classmethod
    @property
    def lang_key(cls) -> str:
        assert cls.MS_INDEX_LANG_KEY is not None, f"MODEL in {cls.__name__} don't set!"
        return cls.MS_INDEX_LANG_KEY

    @classmethod
    @property
    def index_name(cls) -> str:
        assert cls.MS_INDEX_NAME is not None, f"MODEL in {cls.__name__} don't set!"
        return cls.MS_INDEX_NAME

    @classmethod
    def get_allowed_langs_filter(cls, allowed_langs: frozenset[str]) -> list[list[str]]:
        return [[f"{cls.lang_key} = {lang}" for lang in allowed_langs]]

    @classmethod
    def make_request(
        cls, query: str, allowed_langs_filter: list[list[str]], offset: int
    ):
        client = meilisearch.Client(env_config.MEILI_HOST, env_config.MEILI_MASTER_KEY)
        index = client.index(cls.index_name)

        result = index.search(
            query,
            {
                "filter": allowed_langs_filter,
                "offset": offset,
                "limit": 630,
                "attributesToRetrieve": ["id"],
            },
        )

        total: int = result["nbHits"]
        ids: list[int] = [r["id"] for r in result["hits"][:total]]

        return ids

    @classmethod
    async def _get_object_ids(cls, query: SearchQuery) -> list[int]:
        params = cls.get_raw_params()

        allowed_langs_filter = cls.get_allowed_langs_filter(query["allowed_langs"])

        return await asyncio.get_event_loop().run_in_executor(
            cls._executor,
            cls.make_request,
            query["query"],
            allowed_langs_filter,
            params.offset,
        )


class GetRandomService(Generic[MODEL]):
    MODEL_CLASS: Optional[MODEL] = None
    GET_OBJECTS_ID_QUERY: Optional[str] = None
    CUSTOM_CACHE_PREFIX: Optional[str] = None
    CACHE_TTL = 6 * 60 * 60

    @classmethod
    @property
    def model(cls) -> MODEL:
        assert cls.MODEL_CLASS is not None, f"MODEL in {cls.__name__} don't set!"
        return cls.MODEL_CLASS

    @classmethod
    @property
    def database(cls) -> Database:
        return cls.model.Meta.database

    @classmethod
    @property
    def cache_prefix(cls) -> str:
        return cls.CUSTOM_CACHE_PREFIX or cls.model.Meta.tablename

    @staticmethod
    def _get_query_hash(query: frozenset[str]):
        return hash(query)

    @classmethod
    def get_cache_key(cls, query: frozenset[str]) -> str:
        model_class_name = cls.cache_prefix
        query_hash = cls._get_query_hash(query)
        return f"random_{model_class_name}_{query_hash}"

    @classmethod
    @property
    def objects_id_query(cls) -> str:
        assert (
            cls.GET_OBJECTS_ID_QUERY is not None
        ), f"GET_OBJECT_IDS_QUERY in {cls.__name__} don't set!"
        return cls.GET_OBJECTS_ID_QUERY

    @classmethod
    async def _get_objects_from_db(cls, allowed_langs: frozenset[str]) -> list[int]:
        objects = await cls.database.fetch_all(
            cls.objects_id_query, {"langs": allowed_langs}
        )
        return [obj["id"] for obj in objects]

    @classmethod
    async def _get_random_object_from_cache(
        cls, allowed_langs: frozenset[str], redis: aioredis.Redis
    ) -> Optional[int]:
        try:
            key = cls.get_cache_key(allowed_langs)
            active_key = f"{key}_active"

            if not await redis.exists(active_key):
                return None

            data: bytes = await redis.srandmember(key)

            return int(data.decode())
        except aioredis.RedisError as e:
            print(e)
            return None

    @classmethod
    async def _cache_object_ids(
        cls, object_ids: list[int], allowed_langs: frozenset[str], redis: aioredis.Redis
    ) -> bool:
        try:
            key = cls.get_cache_key(allowed_langs)
            active_key = f"{key}_active"

            p = redis.pipeline()

            await p.set(active_key, 1, ex=cls.CACHE_TTL)
            await p.delete(key)
            await p.sadd(key, *object_ids)

            await p.execute()

            return True
        except aioredis.RedisError as e:
            print(e)
            return False

    @classmethod
    async def get_random_id(
        cls,
        allowed_langs: frozenset[str],
        redis: aioredis.Redis,
    ) -> int:
        cached_object_id = await cls._get_random_object_from_cache(allowed_langs, redis)

        if cached_object_id is not None:
            return cached_object_id

        object_ids = await cls._get_objects_from_db(allowed_langs)

        await cls._cache_object_ids(object_ids, allowed_langs, redis)

        return choice(object_ids)


class BaseFilterService(Generic[MODEL, QUERY], BaseSearchService[MODEL, QUERY]):
    @classmethod
    async def _get_object_ids(cls, query: QUERY) -> list[int]:
        return (
            await cls.model.objects.filter(**query)
            .fields("id")
            .values_list(flatten=True)
        )
