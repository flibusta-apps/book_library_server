from datetime import date
from typing import Optional

from pydantic import BaseModel

from app.serializers.author import Author
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


class RemoteBook(Book):
    source: BookSource
    remote_id: int


class BookBaseInfo(BaseModel):
    id: int
    available_types: list[str]


class BookDetail(RemoteBook):
    sequences: list[Sequence]
    genres: list[BookGenre]
    is_deleted: bool
    pages: Optional[int]
