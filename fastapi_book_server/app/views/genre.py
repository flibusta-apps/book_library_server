from fastapi import APIRouter, Depends, HTTPException, status, Request

from fastapi_pagination import Params
from fastapi_pagination.ext.ormar import paginate

from app.depends import check_token, get_allowed_langs
from app.filters.genre import get_genre_filter
from app.models import Genre as GenreDB
from app.serializers.genre import Genre
from app.services.genre import GenreMeiliSearchService
from app.utils.pagination import CustomPage


genre_router = APIRouter(
    prefix="/api/v1/genres", tags=["genres"], dependencies=[Depends(check_token)]
)


PREFETCH_RELATED_FIELDS = ["source"]


@genre_router.get("/", response_model=CustomPage[Genre], dependencies=[Depends(Params)])
async def get_genres(genre_filter: dict = Depends(get_genre_filter)):
    return await paginate(
        GenreDB.objects.prefetch_related(PREFETCH_RELATED_FIELDS)
        .filter(**genre_filter)
        .order_by("description")
    )


@genre_router.get("/metas", response_model=list[str])
async def get_genre_metas():
    genres = await GenreDB.objects.fields("meta").values_list(flatten=True)
    return sorted(list(set(genres)))


@genre_router.get("/{id}", response_model=Genre)
async def get_genre(id: int):
    genre = await GenreDB.objects.prefetch_related(PREFETCH_RELATED_FIELDS).get_or_none(
        id=id
    )

    if genre is None:
        raise HTTPException(status.HTTP_404_NOT_FOUND)

    return genre


@genre_router.get(
    "/search/{query}", response_model=CustomPage[Genre], dependencies=[Depends(Params)]
)
async def search_genres(
    query: str,
    request: Request,
    allowed_langs: frozenset[str] = Depends(get_allowed_langs),
):
    return await GenreMeiliSearchService.get(
        {"query": query, "allowed_langs": allowed_langs},
        request.app.state.redis,
    )
