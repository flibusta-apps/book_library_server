from typing import Union

from fastapi import HTTPException, status

from app.models import Author as AuthorDB
from app.models import Book as BookDB
from app.serializers.book import CreateBook, CreateRemoteBook
from app.services.common import TRGMSearchService, GetRandomService


GET_OBJECT_IDS_QUERY = """
SELECT ARRAY(
    WITH filtered_books AS (
        SELECT id, similarity(title, :query) as sml FROM books
        WHERE books.title % :query AND books.is_deleted = 'f'
    )
    SELECT fbooks.id FROM filtered_books as fbooks
    ORDER BY fbooks.sml DESC, fbooks.id
    LIMIT 210
);
"""


class BookTGRMSearchService(TRGMSearchService):
    MODEL_CLASS = BookDB
    PREFETCH_RELATED = ["source", "authors", "translators", "annotations"]
    GET_OBJECT_IDS_QUERY = GET_OBJECT_IDS_QUERY


class BookCreator:
    @classmethod
    def _raise_bad_request(cls):
        raise HTTPException(status.HTTP_404_NOT_FOUND)

    @classmethod
    async def _create_book(cls, data: CreateBook) -> BookDB:
        data_dict = data.dict()

        author_ids = data_dict.pop("authors", [])
        authors = await AuthorDB.objects.filter(id__in=author_ids).all()

        if len(author_ids) != len(authors):
            cls._raise_bad_request()

        book = await BookDB.objects.create(**data_dict)

        for author in authors:
            await book.authors.add(author)

        return book

    @classmethod
    async def _create_remote_book(cls, data: CreateRemoteBook) -> BookDB:
        data_dict = data.dict()

        author_ids = data_dict.pop("remote_authors", [])
        authors = await AuthorDB.objects.filter(
            source__id=data.source, remote_id__in=author_ids
        ).all()

        if len(author_ids) != len(authors):
            cls._raise_bad_request()

        book = await BookDB.objects.create(**data_dict)

        for author in authors:
            await book.authors.add(author)

        return book

    @classmethod
    async def create(cls, data: Union[CreateBook, CreateRemoteBook]) -> BookDB:
        if isinstance(data, CreateBook):
            return await cls._create_book(data)
        if isinstance(data, CreateRemoteBook):
            return await cls._create_remote_book(data)


class GetRandomBookService(GetRandomService):
    MODEL_CLASS = BookDB
