from typing import Annotated, cast

from fastapi import APIRouter, Depends, HTTPException, Request, status

from fastapi_pagination import Page, Params
from fastapi_pagination.ext.ormar import paginate

from app.depends import check_token, get_allowed_langs
from app.models import Book as BookDB
from app.models import Sequence as SequenceDB
from app.serializers.sequence import Book as SequenceBook
from app.serializers.sequence import PageWithSequence, Sequence
from app.services.sequence import GetRandomSequenceService, SequenceMeiliSearchService
from app.utils.transformer import dict_transformer


sequence_router = APIRouter(
    prefix="/api/v1/sequences",
    tags=["sequence"],
    dependencies=[Depends(check_token)],
)


@sequence_router.get("/", response_model=Page[Sequence], dependencies=[Depends(Params)])
async def get_sequences():
    return await paginate(SequenceDB.objects, transformer=dict_transformer)


@sequence_router.get("/random", response_model=Sequence)
async def get_random_sequence(
    request: Request,
    allowed_langs: Annotated[frozenset[str], Depends(get_allowed_langs)],
):
    sequence_id = await GetRandomSequenceService.get_random_id(
        {"allowed_langs": allowed_langs},
        request.app.state.redis,
    )

    if sequence_id is None:
        raise HTTPException(status.HTTP_204_NO_CONTENT)

    return await SequenceDB.objects.get(id=sequence_id)


@sequence_router.get("/{id}", response_model=Sequence)
async def get_sequence(id: int):
    return await SequenceDB.objects.get(id=id)


@sequence_router.get(
    "/{id}/books",
    response_model=PageWithSequence,
    dependencies=[Depends(Params)],
)
async def get_sequence_books(
    id: int, allowed_langs: Annotated[list[str], Depends(get_allowed_langs)]
):
    page: Page[SequenceBook] = await paginate(
        BookDB.objects.prefetch_related(["source"])
        .select_related(["annotations", "authors", "translators"])
        .filter(sequences__id=id, lang__in=allowed_langs, is_deleted=False)
        .order_by("sequences__booksequences__position"),
        transformer=dict_transformer,
    )

    sequence = await SequenceDB.objects.get_or_none(id=id)

    return PageWithSequence(
        items=page.items,
        total=page.total,
        page=page.page,
        size=page.size,
        pages=page.pages,
        parent_item=Sequence.parse_obj(sequence.dict()) if sequence else None,
    )


@sequence_router.get(
    "/{id}/available_types",
    response_model=list[str],
)
async def sequence_available_types(
    id: int, allowed_langs: Annotated[list[str], Depends(get_allowed_langs)]
) -> list[str]:
    books = await BookDB.objects.filter(
        sequence__id=id, lang__in=allowed_langs, is_deleted=False
    ).all()

    file_types: set[str] = set()

    for book in books:
        for file_type in cast(list[str], book.available_types):
            file_types.add(file_type)

    return list(file_types)


@sequence_router.get(
    "/search/{query}",
    response_model=Page[Sequence],
    dependencies=[Depends(Params)],
)
async def search_sequences(
    query: str,
    request: Request,
    allowed_langs: Annotated[frozenset[str], Depends(get_allowed_langs)],
):
    return await SequenceMeiliSearchService.get(
        {"query": query, "allowed_langs": allowed_langs},
        request.app.state.redis,
    )
