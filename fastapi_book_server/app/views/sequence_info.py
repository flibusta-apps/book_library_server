from typing import Union

from fastapi import APIRouter, Depends, HTTPException, status

from fastapi_pagination import Params
from fastapi_pagination.ext.ormar import paginate
from app.utils.pagination import CustomPage

from app.models import Sequence as SequenceInfoDB
from app.serializers.sequence_info import SequenceInfo, CreateSequenceInfo, CreateRemoteSequenceInfo
from app.services.sequence_info import SequenceInfoCreator
from app.depends import check_token


sequence_info_router = APIRouter(
    prefix="/api/v1/sequence_info",
    tags=["sequence_info"],
    dependencies=[Depends(check_token)],
)


@sequence_info_router.get("/", response_model=CustomPage[SequenceInfo], dependencies=[Depends(Params)])
async def get_sequence_infos():
    return await paginate(
        SequenceInfoDB.objects.prefetch_related(["book", "sequence"])
            .select_related(["book__authors", "book__translations", "book__translations__translator"])
    )


@sequence_info_router.get("/{id}", response_model=SequenceInfo)
async def get_sequence_info(id: int):
    sequence_info = SequenceInfoDB.objects.prefetch_related(["book", "sequence"]) \
        .select_related(["book__authors", "book__translations", "book__translations__translator"]) \
        .get_or_none(id=id)

    if sequence_info is None:
        raise HTTPException(status.HTTP_404_NOT_FOUND)

    return sequence_info


@sequence_info_router.post("/", response_model=SequenceInfo)
async def create_sequence_info(data: Union[CreateSequenceInfo, CreateRemoteSequenceInfo]):
    sequence_info = await SequenceInfoCreator.create(data)

    return await SequenceInfoDB.objects.prefetch_related(["book", "sequence"]) \
        .select_related(["book__authors", "book__translations", "book__translations__translator"]) \
        .get(id=sequence_info.id)
