from typing import Optional

from pydantic import BaseModel

from app.serializers.orjson_config import ORJSONConfig


class AuthorAnnotation(BaseModel):
    id: int
    title: str
    text: str
    file: Optional[str]

    class Config(ORJSONConfig):
        pass


class CreateAuthorAnnotation(BaseModel):
    author: int
    title: str
    text: str
    file: Optional[str]

    class Config(ORJSONConfig):
        pass


class UpdateAuthorAnnotation(BaseModel):
    title: str
    text: str
    file: Optional[str]

    class Config(ORJSONConfig):
        pass
