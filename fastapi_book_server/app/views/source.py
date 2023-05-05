from fastapi import APIRouter, Depends

from fastapi_pagination import Page, Params
from fastapi_pagination.ext.ormar import paginate

from app.depends import check_token
from app.models import Source as SourceDB
from app.serializers.source import Source


source_router = APIRouter(
    prefix="/api/v1/sources",
    tags=["source"],
    dependencies=[Depends(check_token)],
)


@source_router.get("", response_model=Page[Source], dependencies=[Depends(Params)])
async def get_sources():
    return await paginate(SourceDB.objects)
