import abc
import asyncio
from concurrent.futures import ThreadPoolExecutor
import hashlib
from random import choice
from typing import Generic, Optional, TypedDict, TypeVar, Union

from databases import Database
from fastapi_pagination.api import resolve_params
from fastapi_pagination.bases import AbstractParams, RawParams
import meilisearch
import orjson
from ormar import Model, QuerySet
from redis import asyncio as aioredis
from sqlalchemy import Table

from app.utils.orjson_default import default as orjson_default
from app.utils.pagination import Page
from core.config import env_config


MODEL = TypeVar("MODEL", bound=Model)
QUERY = TypeVar("QUERY", bound=TypedDict)


class BaseService(Generic[MODEL, QUERY], abc.ABC):
    MODEL_CLASS: Optional[MODEL] = None
    CACHE_PREFIX: str = ""
    CUSTOM_MODEL_CACHE_NAME: Optional[str] = None
    CACHE_TTL = 6 * 60 * 60

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
        return cls.CUSTOM_MODEL_CACHE_NAME or cls.model.Meta.tablename

    @staticmethod
    def _get_query_hash(query: QUERY) -> str:
        json_value = orjson.dumps(query, orjson_default, option=orjson.OPT_SORT_KEYS)
        return hashlib.md5(json_value).hexdigest()

    @classmethod
    def get_cache_key(cls, query: QUERY) -> str:
        model_class_name = cls.cache_prefix
        query_hash = cls._get_query_hash(query)
        cache_key = f"{model_class_name}_{query_hash}"
        return f"{cls.CACHE_PREFIX}_{cache_key}" if cls.CACHE_PREFIX else cache_key

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
            await p.sadd(key, *object_ids)

            await p.execute()

            return True
        except aioredis.RedisError as e:
            print(e)
            return False


class BaseSearchService(Generic[MODEL, QUERY], BaseService[MODEL, QUERY]):
    SELECT_RELATED: Optional[Union[list[str], str]] = None
    PREFETCH_RELATED: Optional[Union[list[str], str]] = None

    @classmethod
    def get_params(cls) -> AbstractParams:
        return resolve_params()

    @classmethod
    def get_raw_params(cls) -> RawParams:
        return resolve_params().to_raw_params()

    @classmethod
    async def _get_object_ids(cls, query: QUERY) -> list[int]:
        ...

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
    async def get_object_ids(
        cls, query: QUERY, redis: aioredis.Redis, no_cache: bool
    ) -> tuple[int, list[int]]:
        params = cls.get_raw_params()

        if not no_cache and (
            cached_object_ids := await cls.get_cached_ids(query, redis, params)
        ):
            return cached_object_ids

        object_ids = await cls._get_object_ids(query)
        limited_object_ids = object_ids[params.offset : params.offset + params.limit]

        if not no_cache and len(object_ids) != 0:
            await cls.cache_object_ids(query, object_ids, redis)

        return len(object_ids), limited_object_ids

    @classmethod
    async def get_limited_objects(
        cls, query: QUERY, redis: aioredis.Redis, no_cache: bool
    ) -> tuple[int, list[MODEL]]:
        count, object_ids = await cls.get_object_ids(query, redis, no_cache)

        queryset: QuerySet[MODEL] = cls.model.objects

        if cls.PREFETCH_RELATED is not None:
            queryset = queryset.prefetch_related(cls.PREFETCH_RELATED)

        if cls.SELECT_RELATED:
            queryset = queryset.select_related(cls.SELECT_RELATED)

        db_objects = await queryset.filter(id__in=object_ids).all()
        return count, sorted(db_objects, key=lambda o: object_ids.index(o.id))

    @classmethod
    async def get(cls, query: QUERY, redis: aioredis.Redis) -> Page[MODEL]:
        no_cache: bool = query.pop("no_cache", False)  # type: ignore

        params = cls.get_params()

        total, objects = await cls.get_limited_objects(query, redis, no_cache)

        return Page.create(items=objects, total=total, params=params)


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
        assert (
            cls.MS_INDEX_LANG_KEY is not None
        ), f"MS_INDEX_LANG_KEY in {cls.__name__} don't set!"
        return cls.MS_INDEX_LANG_KEY

    @classmethod
    @property
    def index_name(cls) -> str:
        assert (
            cls.MS_INDEX_NAME is not None
        ), f"MS_INDEX_NAME in {cls.__name__} don't set!"
        return cls.MS_INDEX_NAME

    @classmethod
    def get_allowed_langs_filter(cls, allowed_langs: frozenset[str]) -> list[str]:
        langs_values = ", ".join(allowed_langs)
        return [f"{cls.lang_key} IN [{langs_values}]"]

    @classmethod
    def make_request(
        cls, query: str, allowed_langs_filter: list[str], offset: int, limit: int
    ) -> tuple[int, list[int]]:
        client = meilisearch.Client(env_config.MEILI_HOST, env_config.MEILI_MASTER_KEY)
        index = client.index(cls.index_name)

        result = index.search(
            query,
            {
                "filter": allowed_langs_filter,
                "offset": offset,
                "limit": limit,
                "attributesToRetrieve": ["id"],
            },
        )

        total: int = result["estimatedTotalHits"]
        ids: list[int] = [r["id"] for r in result["hits"][:total]]

        return total, ids

    @classmethod
    async def _get_object_ids(cls, query: SearchQuery) -> tuple[int, list[int]]:
        params = cls.get_raw_params()
        allowed_langs_filter = cls.get_allowed_langs_filter(query["allowed_langs"])

        return await asyncio.get_event_loop().run_in_executor(
            cls._executor,
            cls.make_request,
            query["query"],
            allowed_langs_filter,
            params.offset,
            params.limit,
        )

    @classmethod
    async def get_object_ids(
        cls, query: SearchQuery, redis: aioredis.Redis, no_cache: bool
    ) -> tuple[int, list[int]]:
        return await cls._get_object_ids(query)


class GetRandomService(Generic[MODEL, QUERY], BaseService[MODEL, QUERY]):
    GET_OBJECTS_ID_QUERY: Optional[str] = None
    CACHE_PREFIX: str = "random"

    @classmethod
    @property
    def objects_id_query(cls) -> str:
        assert (
            cls.GET_OBJECTS_ID_QUERY is not None
        ), f"GET_OBJECT_IDS_QUERY in {cls.__name__} don't set!"
        return cls.GET_OBJECTS_ID_QUERY

    @classmethod
    async def _get_objects_from_db(cls, query: QUERY) -> list[int]:
        objects = await cls.database.fetch_all(
            cls.objects_id_query, {"langs": query["allowed_langs"]}
        )
        return [obj["id"] for obj in objects]

    @classmethod
    async def _get_random_object_from_cache(
        cls, query: QUERY, redis: aioredis.Redis
    ) -> Optional[int]:
        try:
            key = cls.get_cache_key(query)
            active_key = f"{key}_active"

            if not await redis.exists(active_key):
                return None

            data: str = await redis.srandmember(key)  # type: ignore

            return int(data)
        except aioredis.RedisError as e:
            print(e)
            return None

    @classmethod
    async def get_random_id(
        cls,
        query: QUERY,
        redis: aioredis.Redis,
    ) -> int | None:
        cached_object_id = await cls._get_random_object_from_cache(query, redis)

        if cached_object_id is not None:
            return cached_object_id

        object_ids = await cls._get_objects_from_db(query)

        await cls.cache_object_ids(query, object_ids, redis)

        if len(object_ids) == 0:
            return None

        return choice(object_ids)


class BaseFilterService(Generic[MODEL, QUERY], BaseSearchService[MODEL, QUERY]):
    @classmethod
    async def _get_object_ids(cls, query: QUERY) -> list[int]:
        return (
            await cls.model.objects.filter(**query)
            .fields("id")
            .values_list(flatten=True)
        )
