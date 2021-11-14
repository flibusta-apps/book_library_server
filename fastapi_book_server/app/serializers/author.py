from typing import Optional
from datetime import date

from pydantic import BaseModel


class Author(BaseModel):
    id: int

    first_name: str
    last_name: str
    middle_name: Optional[str]


class CreateAuthor(BaseModel):
    source: int
    remote_id: int

    first_name: str
    last_name: str
    middle_name: Optional[str]


class UpdateAuthor(BaseModel):
    first_name: str
    last_name: str
    middle_name: Optional[str]


class AuthorBook(BaseModel):
    id: int
    title: str
    lang: str
    file_type: str


class Translation(BaseModel):
    translator: Author
    position: int


class TranslatedBook(BaseModel):
    id: int
    title: str
    lang: str
    file_type: str
    authors: list[Author]
    translations: list[Translation]
