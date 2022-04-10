from typing import Optional

from pydantic import BaseModel


class BookAnnotation(BaseModel):
    id: int
    title: str
    text: str
    file: Optional[str]
