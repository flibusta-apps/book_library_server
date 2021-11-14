from fastapi import APIRouter, Depends, HTTPException, status

from fastapi_pagination import Params, Page
from fastapi_pagination.ext.ormar import paginate

from app.models import BookAnnotation as BookAnnotationDB
from app.serializers.book_annotation import BookAnnotation, CreateBookAnnotation, UpdateBookAnnotation
from app.depends import check_token


book_annotation_router = APIRouter(
    prefix="/api/v1/book_annotations",
    tags=["book_annotation"],
    dependencies=[Depends(check_token)]
)


@book_annotation_router.get("/", response_model=Page[BookAnnotation], dependencies=[Depends(Params)])
async def get_book_annotations():
    return await paginate(
        BookAnnotationDB.objects
    )


@book_annotation_router.post("/", response_model=BookAnnotation)
async def create_book_annotation(data: CreateBookAnnotation):
    return await BookAnnotationDB.objects.create(
        **data.dict()
    )


@book_annotation_router.get("/{id}", response_model=BookAnnotation)
async def get_book_annotation(id: int):
    annotation = await BookAnnotationDB.objects.get_or_none(id=id)

    if annotation is None:
        raise HTTPException(status.HTTP_404_NOT_FOUND)

    return annotation


@book_annotation_router.put("/{id}", response_model=BookAnnotation)
async def update_book_annotation(id: int, data: UpdateBookAnnotation):
    annotation = await BookAnnotationDB.objects.get_or_none(id=id)

    if annotation is None:
        raise HTTPException(status.HTTP_404_NOT_FOUND)

    annotation.update_from_dict(data.dict())

    return annotation.save()
