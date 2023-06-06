from fastapi import APIRouter, Depends, HTTPException, status

from fastapi_pagination import Page, Params
from fastapi_pagination.ext.ormar import paginate

from app.depends import check_token
from app.models import AuthorAnnotation as AuthorAnnotationDB
from app.serializers.author_annotation import AuthorAnnotation
from app.utils.transformer import dict_transformer


author_annotation_router = APIRouter(
    prefix="/api/v1/author_annotations",
    tags=["author_annotation"],
    dependencies=[Depends(check_token)],
)


@author_annotation_router.get(
    "/", response_model=Page[AuthorAnnotation], dependencies=[Depends(Params)]
)
async def get_author_annotations():
    return await paginate(AuthorAnnotationDB.objects, transformer=dict_transformer)


@author_annotation_router.get("/{id}", response_model=AuthorAnnotation)
async def get_author_annotation(id: int):
    annotation = await AuthorAnnotationDB.objects.get_or_none(id=id)

    if annotation is None:
        raise HTTPException(status.HTTP_404_NOT_FOUND)

    return annotation
