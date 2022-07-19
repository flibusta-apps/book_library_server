from fastapi import APIRouter, Depends, Request

from fastapi_pagination import Params
from fastapi_pagination.ext.ormar import paginate

from app.depends import check_token, get_allowed_langs
from app.models import Book as BookDB
from app.models import Sequence as SequenceDB
from app.serializers.sequence import Book as SequenceBook
from app.serializers.sequence import Sequence
from app.services.sequence import SequenceMeiliSearchService, GetRandomSequenceService
from app.utils.pagination import CustomPage


sequence_router = APIRouter(
    prefix="/api/v1/sequences",
    tags=["sequence"],
    dependencies=[Depends(check_token)],
)


@sequence_router.get(
    "/", response_model=CustomPage[Sequence], dependencies=[Depends(Params)]
)
async def get_sequences():
    return await paginate(SequenceDB.objects)


@sequence_router.get("/random", response_model=Sequence)
async def get_random_sequence(
    request: Request,
    allowed_langs: frozenset[str] = Depends(get_allowed_langs),
):
    sequence_id = await GetRandomSequenceService.get_random_id(
        {"allowed_langs": allowed_langs},
        request.app.state.redis,
    )

    return await SequenceDB.objects.get(id=sequence_id)


@sequence_router.get("/{id}", response_model=Sequence)
async def get_sequence(id: int):
    return await SequenceDB.objects.get(id=id)


@sequence_router.get(
    "/{id}/books",
    response_model=CustomPage[SequenceBook],
    dependencies=[Depends(Params)],
)
async def get_sequence_books(
    id: int, allowed_langs: list[str] = Depends(get_allowed_langs)
):
    return await paginate(
        BookDB.objects.prefetch_related(["source"])
        .select_related(["annotations", "authors", "translators"])
        .filter(sequences__id=id, lang__in=allowed_langs, is_deleted=False)
        .order_by("sequences__booksequences__position")
    )


@sequence_router.get(
    "/search/{query}",
    response_model=CustomPage[Sequence],
    dependencies=[Depends(Params)],
)
async def search_sequences(
    query: str,
    request: Request,
    allowed_langs: frozenset[str] = Depends(get_allowed_langs),
):
    return await SequenceMeiliSearchService.get(
        {"query": query, "allowed_langs": allowed_langs},
        request.app.state.redis,
    )
