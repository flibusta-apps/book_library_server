from fastapi import APIRouter, Depends

from fastapi_pagination import Params, Page
from fastapi_pagination.ext.ormar import paginate

from app.depends import check_token
from app.models import Source as SourceDB
from app.serializers.source import Source, CreateSource


source_router = APIRouter(
    prefix="/api/v1/sources",
    tags=["source"],
    dependencies=[Depends(check_token)],
)


@source_router.get("", response_model=Page[Source], dependencies=[Depends(Params)])
async def get_sources():
    return await paginate(SourceDB.objects)


@source_router.post("", response_model=Source)
async def create_source(data: CreateSource):
    return await SourceDB.objects.create(**data.dict())
