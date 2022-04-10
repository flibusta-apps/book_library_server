from app.models import Book as BookDB
from app.services.common import (
    TRGMSearchService,
    MeiliSearchService,
    GetRandomService,
    BaseFilterService,
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


class GetRandomBookService(GetRandomService):
    MODEL_CLASS = BookDB
    GET_OBJECTS_ID_QUERY = GET_OBJECTS_ID_QUERY


class BookMeiliSearchService(MeiliSearchService):
    MODEL_CLASS = BookDB
    PREFETCH_RELATED = ["source"]
    SELECT_RELATED = ["authors", "translators", "annotations"]

    MS_INDEX_NAME = "books"
    MS_INDEX_LANG_KEY = "lang"
