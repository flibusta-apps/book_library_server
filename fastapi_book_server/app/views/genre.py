from fastapi import APIRouter, Depends, HTTPException, status

from fastapi_pagination import Params, Page
from fastapi_pagination.ext.ormar import paginate

from app.depends import check_token
from app.models import Genre as GenreDB
from app.serializers.genre import Genre


genre_router = APIRouter(
    prefix="/api/v1/genres", tags=["genres"], dependencies=[Depends(check_token)]
)


PREFETCH_RELATED_FIELDS = ["source"]


@genre_router.get("/", response_model=Page[Genre], dependencies=[Depends(Params)])
async def get_genres():
    return await paginate(GenreDB.objects.prefetch_related(PREFETCH_RELATED_FIELDS))


@genre_router.get("/{id}", response_model=Genre)
async def get_genre(id: int):
    genre = await GenreDB.objects.prefetch_related(PREFETCH_RELATED_FIELDS).get_or_none(
        id=id
    )

    if genre is None:
        raise HTTPException(status.HTTP_404_NOT_FOUND)

    return genre
