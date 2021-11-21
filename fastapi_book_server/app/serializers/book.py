from datetime import date

from pydantic import BaseModel

from app.serializers.author import Author
from app.serializers.orjson_config import ORJSONConfig


class BookSource(BaseModel):
    id: int
    name: str


class Book(BaseModel):
    id: int
    title: str
    lang: str
    file_type: str
    available_types: list[str]
    uploaded: date
    authors: list[Author]

    class Config(ORJSONConfig):
        pass


class BookWithSource(Book):
    source: BookSource


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
