from random import choice as random_choice

from fastapi import APIRouter, Depends, Request

from fastapi_pagination import Params
from fastapi_pagination.ext.ormar import paginate
from app.utils.pagination import CustomPage

from app.models import Sequence as SequenceDB, Book as BookDB, BookSequences as BookSequencesDB
from app.serializers.sequence import Sequence, CreateSequence, Book as SequenceBook
from app.services.sequence import SequenceTGRMSearchService
from app.depends import check_token


sequence_router = APIRouter(
    prefix="/api/v1/sequences",
    tags=["sequence"],
    dependencies=[Depends(check_token)],
)


@sequence_router.get("/", response_model=CustomPage[Sequence], dependencies=[Depends(Params)])
async def get_sequences():
    return await paginate(
        SequenceDB.objects
    )


@sequence_router.get("/random", response_model=Sequence)
async def get_random_sequence():
    sequence_ids: list[int] = await SequenceDB.objects.values_list("id", flatten=True)

    sequence_id = random_choice(sequence_ids)

    return await SequenceDB.objects.get(id=sequence_id)


@sequence_router.get("/{id}", response_model=Sequence)
async def get_sequence(id: int):
    return await SequenceDB.objects.get(id=id)


@sequence_router.get("/{id}/books", response_model=CustomPage[SequenceBook], dependencies=[Depends(Params)])
async def get_sequence_books(id: int):
    return await paginate(
        BookDB.objects.select_related(["source", "annotations", "authors", "translators"])
        .filter(sequences__id=id).order_by("sequences__booksequences__position")
    )


@sequence_router.post("/", response_model=Sequence)
async def create_sequence(data: CreateSequence):
    return await SequenceDB.objects.create(
        **data.dict()
    )


@sequence_router.get("/search/{query}", response_model=CustomPage[Sequence], dependencies=[Depends(Params)])
async def search_sequences(query: str, request: Request):
    return await SequenceTGRMSearchService.get(query, request.app.state.redis)
