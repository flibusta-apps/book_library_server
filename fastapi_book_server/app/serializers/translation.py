from pydantic import BaseModel


class TranslationBook(BaseModel):
    id: int
    title: str
    lang: str
    file_type: str


class TranslationTranslator(BaseModel):
    id: int
    first_name: str
    last_name: str
    middle_name: str


class Translation(BaseModel):
    book: TranslationBook
    translator: TranslationTranslator
    position: int


class CreateTranslation(BaseModel):
    book: int
    translator: int
    position: int


class CreateRemoteTranslation(BaseModel):
    source: int
    remote_book: int
    remote_translator: int
    position: int
