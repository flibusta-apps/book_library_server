from datetime import date
from typing import Optional

from pydantic import BaseModel


class Author(BaseModel):
    id: int

    first_name: str
    last_name: str
    middle_name: Optional[str]

    annotation_exists: bool


class AuthorBook(BaseModel):
    id: int
    title: str
    lang: str
    file_type: str
    available_types: list[str]
    uploaded: date
    translators: list[Author]
    annotation_exists: bool


class TranslatedBook(BaseModel):
    id: int
    title: str
    lang: str
    file_type: str
    available_types: list[str]
    authors: list[Author]
    annotation_exists: bool
