from pydantic import BaseModel

from app.serializers.orjson_config import ORJSONConfig


class SequenceBookAuthor(BaseModel):
    id: int
    first_name: str
    last_name: str
    middle_name: str

    class Config(ORJSONConfig):
        pass


class SeqTranslationTranslator(BaseModel):
    id: int
    first_name: str
    last_name: str
    middle_name: str

    class Config(ORJSONConfig):
        pass


class SequenceBookTranslation(BaseModel):
    id: int
    translator: SeqTranslationTranslator

    class Config(ORJSONConfig):
        pass


class SequenceBook(BaseModel):
    id: int
    title: str
    lang: str
    file_type: str
    authors: SequenceBookAuthor
    translation: SequenceBookTranslation

    class Config(ORJSONConfig):
        pass


class Sequence(BaseModel):
    id: int
    name: str

    class Config(ORJSONConfig):
        pass


class SequenceInfo(BaseModel):
    id: int
    book: SequenceBook
    sequence: Sequence
    position: int

    class Config(ORJSONConfig):
        pass


class CreateSequenceInfo(BaseModel):
    book: int
    sequence: int
    position: int

    class Config(ORJSONConfig):
        pass


class CreateRemoteSequenceInfo(BaseModel):
    source: int
    remote_book: int
    remote_sequence: int
    position: int

    class Config(ORJSONConfig):
        pass
