from typing import Optional

from pydantic import BaseModel

from app.serializers.orjson_config import ORJSONConfig


class BookAnnotation(BaseModel):
    id: int
    title: str
    text: str
    file: Optional[str]

    class Config(ORJSONConfig):
        pass


class CreateBookAnnotation(BaseModel):
    id: int
    title: str
    text: str
    file: Optional[str]

    class Config(ORJSONConfig):
        pass


class UpdateBookAnnotation(BaseModel):
    id: int
    title: str
    text: str
    file: Optional[str]

    class Config(ORJSONConfig):
        pass
