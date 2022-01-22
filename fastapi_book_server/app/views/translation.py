from typing import Union

from fastapi import APIRouter, Depends, HTTPException, status

from fastapi_pagination import Params
from fastapi_pagination.ext.ormar import paginate

from app.depends import check_token
from app.models import Translation as TranslationDB
from app.serializers.translation import (
    Translation,
    CreateTranslation,
    CreateRemoteTranslation,
)
from app.services.translation import TranslationCreator
from app.utils.pagination import CustomPage


translation_router = APIRouter(
    prefix="/api/v1/translation",
    tags=["translation"],
    dependencies=[Depends(check_token)],
)


@translation_router.get(
    "/", response_model=CustomPage[Translation], dependencies=[Depends(Params)]
)
async def get_translations():
    return await paginate(TranslationDB.objects.select_related(["book", "author"]))


@translation_router.post("/", response_model=Translation)
async def create_translation(data: Union[CreateTranslation, CreateRemoteTranslation]):
    translation = await TranslationCreator.create(data)

    return await TranslationDB.objects.select_related(["book", "author"]).get(
        id=translation.id
    )


@translation_router.delete("/{id}", response_model=Translation)
async def delete_translation(id: int):
    translation = await TranslationDB.objects.select_related(
        ["book", "author"]
    ).get_or_none(id=id)

    if translation is None:
        raise HTTPException(status.HTTP_404_NOT_FOUND)

    await translation.delete()

    return translation
