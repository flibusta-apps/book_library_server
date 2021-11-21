from pydantic import BaseModel

from app.serializers.orjson_config import ORJSONConfig


class TranslationBook(BaseModel):
    id: int
    title: str
    lang: str
    file_type: str

    class Config(ORJSONConfig):
        pass


class TranslationTranslator(BaseModel):
    id: int
    first_name: str
    last_name: str
    middle_name: str

    class Config(ORJSONConfig):
        pass


class Translation(BaseModel):
    book: TranslationBook
    translator: TranslationTranslator
    position: int

    class Config(ORJSONConfig):
        pass


class CreateTranslation(BaseModel):
    book: int
    translator: int
    position: int

    class Config(ORJSONConfig):
        pass


class CreateRemoteTranslation(BaseModel):
    source: int
    remote_book: int
    remote_translator: int
    position: int

    class Config(ORJSONConfig):
        pass
