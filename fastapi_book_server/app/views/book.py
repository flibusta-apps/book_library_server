from typing import Union

from fastapi import APIRouter, Depends, Request, HTTPException, status

from fastapi_pagination import Params
from fastapi_pagination.ext.ormar import paginate

from app.depends import check_token, get_allowed_langs
from app.filters.book import get_book_filter
from app.models import Author as AuthorDB
from app.models import Book as BookDB
from app.models import BookAnnotation as BookAnnotationDB
from app.serializers.book import (
    Book,
    RemoteBook,
    BookDetail,
    CreateBook,
    UpdateBook,
    CreateRemoteBook,
)
from app.serializers.book_annotation import BookAnnotation
from app.services.book import BookTGRMSearchService, GetRandomBookService, BookCreator
from app.utils.pagination import CustomPage


book_router = APIRouter(
    prefix="/api/v1/books",
    tags=["book"],
    dependencies=[Depends(check_token)],
)

SELECT_RELATED_FIELDS = ["source"]
PREFETCH_RELATED_FIELDS = ["authors", "translators", "annotations"]


@book_router.get(
    "/", response_model=CustomPage[RemoteBook], dependencies=[Depends(Params)]
)
async def get_books(book_filter: dict = Depends(get_book_filter)):
    return await paginate(
        BookDB.objects.select_related(SELECT_RELATED_FIELDS)
        .prefetch_related(PREFETCH_RELATED_FIELDS)
        .filter(**book_filter)
    )


@book_router.post("/", response_model=Book)
async def create_book(data: Union[CreateBook, CreateRemoteBook]):
    book = await BookCreator.create(data)

    return (
        await BookDB.objects.select_related(SELECT_RELATED_FIELDS)
        .prefetch_related(PREFETCH_RELATED_FIELDS)
        .get(id=book.id)
    )


@book_router.get("/random", response_model=BookDetail)
async def get_random_book(allowed_langs: list[str] = Depends(get_allowed_langs)):
    book_id = await GetRandomBookService.get_random_id(allowed_langs)

    return (
        await BookDB.objects.select_related(SELECT_RELATED_FIELDS)
        .prefetch_related(PREFETCH_RELATED_FIELDS)
        .get(id=book_id)
    )


@book_router.get("/{id}", response_model=BookDetail)
async def get_book(id: int):
    book = (
        await BookDB.objects.select_related(SELECT_RELATED_FIELDS)
        .prefetch_related(PREFETCH_RELATED_FIELDS)
        .get_or_none(id=id)
    )

    if book is None:
        raise HTTPException(status.HTTP_404_NOT_FOUND)

    return book


@book_router.get("/remote/{source_id}/{remote_id}", response_model=Book)
async def get_remote_book(source_id: int, remote_id: int):
    book = (
        await BookDB.objects.select_related(SELECT_RELATED_FIELDS)
        .prefetch_related(PREFETCH_RELATED_FIELDS)
        .get_or_none(source=source_id, remote_id=remote_id)
    )

    if book is None:
        raise HTTPException(status.HTTP_404_NOT_FOUND)

    return book


@book_router.put("/{id}", response_model=Book)
async def update_book(id: int, data: UpdateBook):
    book = (
        await BookDB.objects.select_related(SELECT_RELATED_FIELDS)
        .prefetch_related(PREFETCH_RELATED_FIELDS)
        .get_or_none(id=id)
    )

    if book is None:
        raise HTTPException(status.HTTP_404_NOT_FOUND)

    for author in list(book.authors):
        await book.authors.remove(author)

    data_dict = data.dict()

    author_ids = data_dict.pop("authors", [])
    authors = await AuthorDB.objects.filter(id__in=author_ids).all()

    book = await BookDB.objects.create(**data_dict)

    for author in authors:
        await book.authors.add(author)

    return book


@book_router.get("/{id}/annotation", response_model=BookAnnotation)
async def get_book_annotation(id: int):
    annotation = await BookAnnotationDB.objects.get(book__id=id)

    if annotation is None:
        raise HTTPException(status.HTTP_404_NOT_FOUND)

    return annotation


@book_router.get(
    "/search/{query}", response_model=CustomPage[Book], dependencies=[Depends(Params)]
)
async def search_books(
    query: str, request: Request, allowed_langs: list[str] = Depends(get_allowed_langs)
):
    return await BookTGRMSearchService.get(
        query, request.app.state.redis, allowed_langs
    )
