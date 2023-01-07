from fastapi import APIRouter, Depends, HTTPException, status
from fastapi_pagination import Page, Params
from fastapi_pagination.ext.ormar import paginate

from app.depends import check_token
from app.models import BookAnnotation as BookAnnotationDB
from app.serializers.book_annotation import BookAnnotation

book_annotation_router = APIRouter(
    prefix="/api/v1/book_annotations",
    tags=["book_annotation"],
    dependencies=[Depends(check_token)],
)


@book_annotation_router.get(
    "/", response_model=Page[BookAnnotation], dependencies=[Depends(Params)]
)
async def get_book_annotations():
    return await paginate(BookAnnotationDB.objects)


@book_annotation_router.get("/{id}", response_model=BookAnnotation)
async def get_book_annotation(id: int):
    annotation = await BookAnnotationDB.objects.get_or_none(id=id)

    if annotation is None:
        raise HTTPException(status.HTTP_404_NOT_FOUND)

    return annotation
