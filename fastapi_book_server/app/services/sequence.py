from app.models import Sequence
from app.services.common import TRGMSearchService, GetRandomService


GET_OBJECT_IDS_QUERY = """
SELECT ARRAY (
    WITH filtered_sequences AS (
        SELECT
            id,
            similarity(name, :query) as sml,
            (
                SELECT count(*) FROM book_sequences
                LEFT JOIN books
                ON (books.id = book AND
                    books.is_deleted = 'f' AND
                    books.lang = ANY(:langs ::text[]))
                WHERE sequence = sequences.id
            ) as books_count
        FROM sequences
        WHERE name % :query AND
        EXISTS (
            SELECT * FROM book_sequences
            LEFT JOIN books
            ON (books.id = book AND
                books.is_deleted = 'f' AND
                books.lang = ANY(:langs ::text[]))
            WHERE sequence = sequences.id
        )
    )
    SELECT fsequences.id FROM filtered_sequences as fsequences
    ORDER BY fsequences.sml DESC, fsequences.books_count DESC
    LIMIT 210
);
"""


class SequenceTGRMSearchService(TRGMSearchService):
    MODEL_CLASS = Sequence
    SELECT_RELATED = ["source"]
    GET_OBJECT_IDS_QUERY = GET_OBJECT_IDS_QUERY


GET_RANDOM_OBJECT_ID_QUERY = """
WITH filtered_sequences AS (
    SELECT id FROM sequences
    WHERE EXISTS (
        SELECT * FROM book_sequences
        LEFT JOIN books
        ON (books.id = book AND
            books.is_deleted = 'f' AND
            books.lang = ANY(:langs ::text[]))
        WHERE sequence = sequences.id
    )
)
SELECT id FROM filtered_sequences
ORDER BY RANDOM() LIMIT 1;
"""


class GetRandomSequenceService(GetRandomService):
    MODEL_CLASS = Sequence
    GET_RANDOM_OBJECT_ID_QUERY = GET_RANDOM_OBJECT_ID_QUERY
