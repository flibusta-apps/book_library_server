from pydantic import BaseModel, constr

from app.serializers.orjson_config import ORJSONConfig


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

    class Config(ORJSONConfig):
        pass


class CreateGenre(BaseModel):
    source: int
    remote_id: int
    code: constr(max_length=45)  # type: ignore
    description: constr(max_length=99)  # type: ignore
    meta: constr(max_length=45)  # type: ignore

    class Config(ORJSONConfig):
        pass


class UpdateGenre(BaseModel):
    source: int
    remote_id: int
    code: constr(max_length=45)  # type: ignore
    description: constr(max_length=99)  # type: ignore
    meta: constr(max_length=45)  # type: ignore

    class Config(ORJSONConfig):
        pass
