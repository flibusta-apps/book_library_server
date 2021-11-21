from typing import Optional

from pydantic import BaseModel

from app.serializers.orjson_config import ORJSONConfig


class Author(BaseModel):
    id: int

    first_name: str
    last_name: str
    middle_name: Optional[str]

    class Config(ORJSONConfig):
        pass


class CreateAuthor(BaseModel):
    source: int
    remote_id: int

    first_name: str
    last_name: str
    middle_name: Optional[str]

    class Config(ORJSONConfig):
        pass


class UpdateAuthor(BaseModel):
    first_name: str
    last_name: str
    middle_name: Optional[str]

    class Config(ORJSONConfig):
        pass


class AuthorBook(BaseModel):
    id: int
    title: str
    lang: str
    file_type: str

    class Config(ORJSONConfig):
        pass


class Translation(BaseModel):
    translator: Author
    position: int

    class Config(ORJSONConfig):
        pass


class TranslatedBook(BaseModel):
    id: int
    title: str
    lang: str
    file_type: str
    authors: list[Author]
    translations: list[Translation]

    class Config(ORJSONConfig):
        pass
