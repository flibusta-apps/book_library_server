from datetime import date

from pydantic import BaseModel

from app.serializers.author import Author
from app.serializers.orjson_config import ORJSONConfig
from app.serializers.sequence import Sequence


class BookSource(BaseModel):
    id: int
    name: str


class BookGenre(BaseModel):
    id: int
    description: str


class Book(BaseModel):
    id: int
    title: str
    lang: str
    file_type: str
    available_types: list[str]
    uploaded: date
    authors: list[Author]
    translators: list[Author]
    annotation_exists: bool

    class Config(ORJSONConfig):
        pass


class RemoteBook(Book):
    source: BookSource
    remote_id: int


class BookDetail(RemoteBook):
    sequences: list[Sequence]
    genres: list[BookGenre]
    is_deleted: bool


class CreateBook(BaseModel):
    source: int
    remote_id: int
    title: str
    lang: str
    file_type: str
    uploaded: date
    authors: list[int]

    class Config(ORJSONConfig):
        pass


class UpdateBook(BaseModel):
    title: str
    lang: str
    file_type: str
    uploaded: date
    authors: list[int]

    class Config(ORJSONConfig):
        pass


class CreateRemoteBook(BaseModel):
    source: int
    remote_id: int
    title: str
    lang: str
    file_type: str
    uploaded: date
    remote_authors: list[int]

    class Config(ORJSONConfig):
        pass
