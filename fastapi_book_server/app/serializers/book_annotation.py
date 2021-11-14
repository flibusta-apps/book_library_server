from typing import Optional

from pydantic import BaseModel


class BookAnnotation(BaseModel):
    id: int
    title: str
    text: str
    file: Optional[str]


class CreateBookAnnotation(BaseModel):
    id: int
    title: str
    text: str
    file: Optional[str]


class UpdateBookAnnotation(BaseModel):
    id: int
    title: str
    text: str
    file: Optional[str]
