from datetime import date
from typing import Optional

from pydantic import BaseModel

from app.serializers.orjson_config import ORJSONConfig


class Sequence(BaseModel):
    id: int
    name: str

    class Config(ORJSONConfig):
        pass


class CreateSequence(BaseModel):
    source: int
    remote_id: int
    name: str

    class Config(ORJSONConfig):
        pass


class Author(BaseModel):
    id: int

    first_name: str
    last_name: str
    middle_name: Optional[str]

    class Config(ORJSONConfig):
        pass


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

    class Config(ORJSONConfig):
        pass
