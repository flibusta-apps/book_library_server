from fastapi import APIRouter, Depends, HTTPException, status

from fastapi_pagination import Params, Page
from fastapi_pagination.ext.ormar import paginate
from fastapi_book_server.app.utils.pagination import CustomPage

from app.models import Author as AuthorDB, AuthorAnnotation as AuthorAnnotationDB, Book as BookDB
from app.serializers.author import Author, CreateAuthor, UpdateAuthor, AuthorBook, TranslatedBook
from app.serializers.author_annotation import AuthorAnnotation
from app.services.author import AuthorTGRMSearchService


author_router = APIRouter(
    prefix="/api/v1/authors",
    tags=["author"],
)


@author_router.get("/", response_model=Page[Author], dependencies=[Depends(Params)])
async def get_authors():
    return await paginate(
        AuthorDB.objects.prefetch_related("source")
    )


@author_router.post("/", response_model=Author)
async def create_author(data: CreateAuthor):
    author = await AuthorDB.objects.create(
        **data.dict()
    )

    return await AuthorDB.objects.prefetch_related("source").get(id=author.id)


@author_router.get("/{id}", response_model=Author)
async def get_author(id: int):
    author = await AuthorDB.objects.prefetch_related("source").get_or_none(id=id)

    if author is None:
        raise HTTPException(status.HTTP_404_NOT_FOUND)

    return author


@author_router.put("/{id}", response_model=Author)
async def update_author(id: int, data: UpdateAuthor):
    author = await AuthorDB.objects.get_or_none(id=id)

    if author is None:
        raise HTTPException(status.HTTP_404_NOT_FOUND)

    author.update_from_dict(data.dict())

    return await author.save()


@author_router.get("/{id}/annotation", response_model=AuthorAnnotation)
async def get_author_annotation(id: int):
    annotation = await AuthorAnnotationDB.objects.get_or_none(author__id=id)

    if annotation is None:
        raise HTTPException(status.HTTP_404_NOT_FOUND)
    
    return annotation


@author_router.get("/{id}/books", response_model=CustomPage[AuthorBook], dependencies=[Depends(Params)])
async def get_author_books(id: int):
    return await paginate(
        BookDB.objects.filter(author__id=id).order_by('title')
    )


@author_router.get("/{id}/translated_books", response_model=CustomPage[TranslatedBook])
async def get_translated_books(id: int):
    return await paginate(
        BookDB.objects.select_related(["translations", "translations__translator"]).filter(translations__translator__id=id)
    )


@author_router.get("/search/{query}", response_model=Page[Author], dependencies=[Depends(Params)])
async def search_authors(query: str):
    return await AuthorTGRMSearchService.get(query)
