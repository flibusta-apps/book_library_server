from typing import Optional

from pydantic import BaseModel


class AuthorAnnotation(BaseModel):
    id: int
    title: str
    text: str
    file: Optional[str]
