from app.models import Author
from app.services.common import MeiliSearchService, TRGMSearchService


GET_OBJECT_IDS_QUERY = """
SELECT ARRAY(
    WITH filtered_authors AS (
        SELECT
            id,
            GREATEST(
                similarity(
                    (last_name || ' ' || first_name || ' ' || middle_name),
                    :query
                ),
                similarity((last_name || ' ' || first_name), :query),
                similarity((last_name), :query)
            ) as sml,
            (
                SELECT count(*) FROM translations
                LEFT JOIN books ON (books.id = book)
                WHERE author = authors.id AND books.is_deleted = 'f' AND
                    books.lang = ANY(:langs ::text[])
            ) as books_count
        FROM authors
        WHERE (
            (last_name || ' ' || first_name || ' ' || middle_name) % :query OR
            (last_name || ' ' || first_name) % :query OR
            (last_name) % :query
        ) AND
        EXISTS (
            SELECT * FROM translations
            LEFT JOIN books ON (books.id = book)
            WHERE author = authors.id AND books.is_deleted = 'f' AND
                books.lang = ANY(:langs ::text[])
        )
    )
    SELECT fauthors.id FROM filtered_authors as fauthors
    ORDER BY fauthors.sml DESC, fauthors.books_count DESC
    LIMIT 210
);
"""


class TranslatorTGRMSearchService(TRGMSearchService):
    MODEL_CLASS = Author
    CUSTOM_CACHE_PREFIX = "translator"
    PREFETCH_RELATED = ["source"]
    SELECT_RELATED = ["annotations"]

    GET_OBJECT_IDS_QUERY = GET_OBJECT_IDS_QUERY


class TranslatorMeiliSearchService(MeiliSearchService):
    MODEL_CLASS = Author
    CUSTOM_CACHE_PREFIX = "translator"
    PREFETCH_RELATED = ["source"]
    SELECT_RELATED = ["annotations"]

    MS_INDEX_NAME = "authors"
    MS_INDEX_LANG_KEY = "translator_langs"
