from pydantic import BaseModel


class SequenceBookAuthor(BaseModel):
    id: int
    first_name: str
    last_name: str
    middle_name: str


class SeqTranslationTranslator(BaseModel):
    id: int
    first_name: str
    last_name: str
    middle_name: str


class SequenceBookTranslation(BaseModel):
    id: int
    translator: SeqTranslationTranslator


class SequenceBook(BaseModel):
    id: int
    title: str
    lang: str
    file_type: str
    authors: SequenceBookAuthor
    translation: SequenceBookTranslation


class Sequence(BaseModel):
    id: int
    name: str


class SequenceInfo(BaseModel):
    id: int
    book: SequenceBook
    sequence: Sequence
    position: int


class CreateSequenceInfo(BaseModel):
    book: int
    sequence: int
    position: int


class CreateRemoteSequenceInfo(BaseModel):
    source: int
    remote_book: int
    remote_sequence: int
    position: int
