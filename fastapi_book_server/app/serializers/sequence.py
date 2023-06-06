from datetime import date
from typing import Optional

from fastapi_pagination import Page
from pydantic import BaseModel


class Sequence(BaseModel):
    id: int
    name: str


class Author(BaseModel):
    id: int

    first_name: str
    last_name: str
    middle_name: Optional[str]

    annotation_exists: bool


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


class PageWithSequence(Page[Book]):
    parent_item: Sequence | None
