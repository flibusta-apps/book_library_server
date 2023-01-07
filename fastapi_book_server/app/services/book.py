from typing import Optional, TypedDict

from app.models import Book as BookDB
from app.services.common import (
    BaseFilterService,
    GetRandomService,
    MeiliSearchService,
    TRGMSearchService,
)

GET_OBJECT_IDS_QUERY = """
SELECT ARRAY(
    WITH filtered_books AS (
        SELECT id, similarity(title, :query) as sml FROM books
        WHERE books.title % :query AND books.is_deleted = 'f'
              AND books.lang = ANY(:langs ::text[])
    )
    SELECT fbooks.id FROM filtered_books as fbooks
    ORDER BY fbooks.sml DESC, fbooks.id
    LIMIT 210
);
"""


class BookTGRMSearchService(TRGMSearchService):
    MODEL_CLASS = BookDB
    PREFETCH_RELATED = ["source"]
    SELECT_RELATED = ["authors", "translators", "annotations"]

    GET_OBJECT_IDS_QUERY = GET_OBJECT_IDS_QUERY


class BookFilterService(BaseFilterService):
    MODEL_CLASS = BookDB
    PREFETCH_RELATED = ["source"]
    SELECT_RELATED = ["authors", "translators", "annotations"]


GET_OBJECTS_ID_QUERY = """
WITH filtered_books AS (
    SELECT id FROM books
    WHERE books.is_deleted = 'f' AND books.lang = ANY(:langs ::text[])
)
SELECT id FROM filtered_books;
"""


GET_OBJECTS_ID_BY_GENRE_QUERY = """
WITH filtered_books AS (
    SELECT books.id FROM books
    LEFT JOIN book_genres ON (book_genres.book = books.id)
    WHERE books.is_deleted = 'f' AND book_genres.genre = :genre
        AND books.lang = ANY(:langs ::text[])
)
SELECT id FROM filtered_books;
"""


class RandomBookServiceQuery(TypedDict):
    genre: Optional[int]
    allowed_langs: frozenset[str]


class GetRandomBookService(GetRandomService[BookDB, RandomBookServiceQuery]):
    MODEL_CLASS = BookDB  # type: ignore
    PREFETCH_RELATED = ["source"]
    SELECT_RELATED = ["authors", "translators", "annotations"]

    GET_OBJECTS_ID_QUERY = GET_OBJECTS_ID_QUERY
    GET_OBJECTS_ID_BY_GENRE_QUERY = GET_OBJECTS_ID_BY_GENRE_QUERY

    @classmethod
    async def _get_objects_from_db(cls, query: RandomBookServiceQuery) -> list[int]:
        if query.get("genre") is None:
            ex_query = cls.objects_id_query
            params = {"langs": query["allowed_langs"]}
        else:
            ex_query = cls.GET_OBJECTS_ID_BY_GENRE_QUERY
            params = {"langs": query["allowed_langs"], "genre": query["genre"]}

        objects = await cls.database.fetch_all(ex_query, params)
        return [obj["id"] for obj in objects]


class BookMeiliSearchService(MeiliSearchService):
    MODEL_CLASS = BookDB
    PREFETCH_RELATED = ["source"]
    SELECT_RELATED = ["authors", "translators", "annotations"]

    MS_INDEX_NAME = "books"
    MS_INDEX_LANG_KEY = "lang"
