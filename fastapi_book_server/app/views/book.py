from typing import Optional

from fastapi import APIRouter, Depends, HTTPException, Request, status

from fastapi_pagination import Params

from app.depends import check_token, get_allowed_langs
from app.filters.book import get_book_filter
from app.models import Book as BookDB
from app.models import BookAnnotation as BookAnnotationDB
from app.serializers.book import Book, BookBaseInfo, BookDetail, RemoteBook
from app.serializers.book_annotation import BookAnnotation
from app.services.book import (
    BookBaseInfoFilterService,
    BookFilterService,
    BookMeiliSearchService,
    GetRandomBookService,
)
from app.utils.pagination import Page


book_router = APIRouter(
    prefix="/api/v1/books",
    tags=["book"],
    dependencies=[Depends(check_token)],
)

PREFETCH_RELATED_FIELDS = ["source"]
SELECT_RELATED_FIELDS = ["authors", "translators", "annotations"]

DETAIL_SELECT_RELATED_FIELDS = ["sequences", "genres"]


@book_router.get("/", response_model=Page[RemoteBook], dependencies=[Depends(Params)])
async def get_books(
    request: Request,
    book_filter: dict = Depends(get_book_filter),
):
    return await BookFilterService.get(book_filter, request.app.state.redis)


@book_router.get(
    "/base/", response_model=Page[BookBaseInfo], dependencies=[Depends(Params)]
)
async def get_base_books_info(
    request: Request, book_filter: dict = Depends(get_book_filter)
):
    return await BookBaseInfoFilterService.get(book_filter, request.app.state.redis)


@book_router.get("/last", response_model=int)
async def get_last_book_id():
    book = await BookDB.objects.order_by("-id").first()
    return book.id


@book_router.get("/random", response_model=BookDetail)
async def get_random_book(
    request: Request,
    allowed_langs: frozenset[str] = Depends(get_allowed_langs),
    genre: Optional[int] = None,
):
    book_id = await GetRandomBookService.get_random_id(
        {"allowed_langs": allowed_langs, "genre": genre}, request.app.state.redis
    )

    if book_id is None:
        raise HTTPException(status.HTTP_204_NO_CONTENT)

    book = (
        await BookDB.objects.select_related(
            SELECT_RELATED_FIELDS + DETAIL_SELECT_RELATED_FIELDS
        )
        .prefetch_related(PREFETCH_RELATED_FIELDS)
        .get(id=book_id)
    )

    return book


@book_router.get("/{id}", response_model=BookDetail)
async def get_book(id: int):
    book = (
        await BookDB.objects.select_related(
            SELECT_RELATED_FIELDS + DETAIL_SELECT_RELATED_FIELDS
        )
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


@book_router.get("/{id}/annotation", response_model=BookAnnotation)
async def get_book_annotation(id: int):
    annotation = await BookAnnotationDB.objects.get(book__id=id)

    if annotation is None:
        raise HTTPException(status.HTTP_404_NOT_FOUND)

    return annotation


@book_router.get(
    "/search/{query}", response_model=Page[Book], dependencies=[Depends(Params)]
)
async def search_books(
    query: str,
    request: Request,
    allowed_langs: frozenset[str] = Depends(get_allowed_langs),
):
    return await BookMeiliSearchService.get(
        {"query": query, "allowed_langs": allowed_langs},
        request.app.state.redis,
    )
