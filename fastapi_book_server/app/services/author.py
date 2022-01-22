from app.models import Author
from app.services.common import TRGMSearchService, GetRandomService


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
                SELECT count(*) FROM book_authors
                LEFT JOIN books
                ON (books.id = book AND
                    books.is_deleted = 'f' AND
                    books.lang = ANY(:langs ::text[]))
                WHERE author = authors.id
            ) as books_count
        FROM authors
        WHERE (
            (last_name || ' ' || first_name || ' ' || middle_name) % :query OR
            (last_name || ' ' || first_name) % :query OR
            (last_name) % :query
        ) AND
        EXISTS (
            SELECT * FROM book_authors
            LEFT JOIN books
            ON (books.id = book AND
                books.is_deleted = 'f' AND
                books.lang = ANY(:langs ::text[]))
            WHERE author = authors.id
        )
    )
    SELECT fauthors.id FROM filtered_authors as fauthors
    ORDER BY fauthors.sml DESC, fauthors.books_count DESC
    LIMIT 210
);
"""


class AuthorTGRMSearchService(TRGMSearchService):
    MODEL_CLASS = Author
    SELECT_RELATED = ["source"]
    PREFETCH_RELATED = ["annotations"]
    GET_OBJECT_IDS_QUERY = GET_OBJECT_IDS_QUERY


GET_RANDOM_OBJECT_ID_QUERY = """
WITH filtered_authors AS (
    SELECT id FROM authors
    WHERE EXISTS (
        SELECT * FROM book_authors
        LEFT JOIN books
        ON (books.id = book AND
            books.is_deleted = 'f' AND
            books.lang = ANY(:langs ::text[]))
        WHERE author = authors.id
    )
)
SELECT id FROM filtered_authors
ORDER BY RANDOM() LIMIT 1;
"""


class GetRandomAuthorService(GetRandomService):
    MODEL_CLASS = Author
    GET_RANDOM_OBJECT_ID_QUERY = GET_RANDOM_OBJECT_ID_QUERY
