from typing import Optional

from pydantic import BaseModel


class AuthorAnnotation(BaseModel):
    id: int
    title: str
    text: str
    file: Optional[str]


class CreateAuthorAnnotation(BaseModel):
    author: int
    title: str
    text: str
    file: Optional[str]


class UpdateAuthorAnnotation(BaseModel):
    title: str
    text: str
    file: Optional[str]
