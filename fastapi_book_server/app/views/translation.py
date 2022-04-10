from fastapi import APIRouter, Depends

from fastapi_pagination import Params
from fastapi_pagination.ext.ormar import paginate

from app.depends import check_token
from app.models import Translation as TranslationDB
from app.serializers.translation import Translation
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
