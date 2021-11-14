from datetime import date

from pydantic import BaseModel

from app.serializers.author import Author


class Book(BaseModel):
    id: int
    title: str
    lang: str
    file_type: str
    uploaded: date
    authors: list[Author]


class CreateBook(BaseModel):
    source: int
    remote_id: int
    title: str
    lang: str
    file_type: str
    uploaded: date
    authors: list[int]


class UpdateBook(BaseModel):
    title: str
    lang: str
    file_type: str
    uploaded: date
    authors: list[int]


class CreateRemoteBook(BaseModel):
    source: int
    remote_id: int
    title: str
    lang: str
    file_type: str
    uploaded: date
    remote_authors: list[int]
