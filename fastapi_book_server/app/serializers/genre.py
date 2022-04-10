from pydantic import BaseModel


class GenreSource(BaseModel):
    id: int
    name: str


class Genre(BaseModel):
    id: int
    source: GenreSource
    remote_id: int
    code: str
    description: str
    meta: str
