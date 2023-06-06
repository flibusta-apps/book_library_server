from datetime import date
from typing import Optional

from fastapi_pagination import Page
from pydantic import BaseModel

from app.serializers.sequence import Sequence


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
    sequences: list[Sequence]
    annotation_exists: bool


class TranslatedBook(BaseModel):
    id: int
    title: str
    lang: str
    file_type: str
    available_types: list[str]
    authors: list[Author]
    sequences: list[Sequence]
    annotation_exists: bool


class PageWithAuthorBook(Page[AuthorBook]):
    parent_item: Author | None


class PageWithTranslatedBook(Page[TranslatedBook]):
    parent_item: TranslatedBook | None
