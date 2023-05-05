from fastapi import APIRouter, Depends, HTTPException, Request, status

from fastapi_pagination import Params
from fastapi_pagination.ext.ormar import paginate

from app.depends import check_token, get_allowed_langs
from app.models import Author as AuthorDB
from app.models import AuthorAnnotation as AuthorAnnotationDB
from app.models import Book as BookDB
from app.serializers.author import Author, AuthorBook, TranslatedBook
from app.serializers.author_annotation import AuthorAnnotation
from app.services.author import AuthorMeiliSearchService, GetRandomAuthorService
from app.services.translator import TranslatorMeiliSearchService
from app.utils.pagination import Page


author_router = APIRouter(
    prefix="/api/v1/authors",
    tags=["author"],
    dependencies=[Depends(check_token)],
)


PREFETCH_RELATED_FIELDS = ["source"]
SELECT_RELATED_FIELDS = ["annotations"]


@author_router.get("/", response_model=Page[Author], dependencies=[Depends(Params)])
async def get_authors():
    return await paginate(
        AuthorDB.objects.select_related(SELECT_RELATED_FIELDS).prefetch_related(
            PREFETCH_RELATED_FIELDS
        )
    )


@author_router.get("/random", response_model=Author)
async def get_random_author(
    request: Request,
    allowed_langs: frozenset[str] = Depends(get_allowed_langs),
):
    author_id = await GetRandomAuthorService.get_random_id(
        {"allowed_langs": allowed_langs}, request.app.state.redis
    )

    return (
        await AuthorDB.objects.select_related(SELECT_RELATED_FIELDS)
        .prefetch_related(PREFETCH_RELATED_FIELDS)
        .get(id=author_id)
    )


@author_router.get("/{id}", response_model=Author)
async def get_author(id: int):
    author = (
        await AuthorDB.objects.select_related(SELECT_RELATED_FIELDS)
        .prefetch_related(PREFETCH_RELATED_FIELDS)
        .get_or_none(id=id)
    )

    if author is None:
        raise HTTPException(status.HTTP_404_NOT_FOUND)

    return author


@author_router.get("/{id}/annotation", response_model=AuthorAnnotation)
async def get_author_annotation(id: int):
    annotation = await AuthorAnnotationDB.objects.get_or_none(author__id=id)

    if annotation is None:
        raise HTTPException(status.HTTP_404_NOT_FOUND)

    return annotation


@author_router.get(
    "/{id}/books", response_model=Page[AuthorBook], dependencies=[Depends(Params)]
)
async def get_author_books(
    id: int, allowed_langs: list[str] = Depends(get_allowed_langs)
):
    return await paginate(
        BookDB.objects.prefetch_related(["source"])
        .select_related(["annotations", "translators"])
        .filter(authors__id=id, lang__in=allowed_langs, is_deleted=False)
        .order_by("title")
    )


@author_router.get(
    "/search/{query}", response_model=Page[Author], dependencies=[Depends(Params)]
)
async def search_authors(
    query: str,
    request: Request,
    allowed_langs: frozenset[str] = Depends(get_allowed_langs),
):
    return await AuthorMeiliSearchService.get(
        {"query": query, "allowed_langs": allowed_langs},
        request.app.state.redis,
    )


translator_router = APIRouter(
    prefix="/api/v1/translators",
    tags=["author"],
    dependencies=[Depends(check_token)],
)


@translator_router.get("/{id}/books", response_model=Page[TranslatedBook])
async def get_translated_books(
    id: int, allowed_langs: list[str] = Depends(get_allowed_langs)
):
    return await paginate(
        BookDB.objects.prefetch_related(["source"])
        .select_related(["annotations", "authors"])
        .filter(
            translators__id=id,
            lang__in=allowed_langs,
            is_deleted=False,
        )
    )


@translator_router.get(
    "/search/{query}", response_model=Page[Author], dependencies=[Depends(Params)]
)
async def search_translators(
    query: str,
    request: Request,
    allowed_langs: frozenset[str] = Depends(get_allowed_langs),
):
    return await TranslatorMeiliSearchService.get(
        {"query": query, "allowed_langs": allowed_langs},
        request.app.state.redis,
    )
