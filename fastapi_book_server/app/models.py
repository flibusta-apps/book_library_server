from datetime import date

import ormar

from core.db import metadata, database


class BaseMeta(ormar.ModelMeta):
    metadata = metadata
    database = database


class Source(ormar.Model):
    class Meta(BaseMeta):
        tablename = "sources"

    id: int = ormar.SmallInteger(primary_key=True, nullable=False)  # type: ignore

    name: str = ormar.String(max_length=32, nullable=False, unique=True)  # type: ignore


class Author(ormar.Model):
    class Meta(BaseMeta):
        tablename = "authors"
        constraints = [
            ormar.UniqueColumns("source", "remote_id"),
        ]

    id: int = ormar.Integer(primary_key=True, nullable=False)  # type: ignore

    source: Source = ormar.ForeignKey(Source, nullable=False)
    remote_id: int = ormar.Integer(minimum=0, nullable=False)  # type: ignore

    first_name: str = ormar.String(max_length=256, nullable=False)  # type: ignore
    last_name: str = ormar.String(max_length=256, nullable=False)  # type: ignore
    middle_name: str = ormar.String(max_length=256, nullable=True, default="")  # type: ignore


class AuthorAnnotation(ormar.Model):
    class Meta(BaseMeta):
        tablename = "author_annotations"

    id = ormar.Integer(primary_key=True, nullable=False)

    author: Author = ormar.ForeignKey(Author, nullable=False, unique=True)

    title: str = ormar.String(max_length=256, nullable=False, default="")  # type: ignore
    text: str = ormar.Text(nullable=False, default="")  # type: ignore
    file: str = ormar.String(max_length=256, nullable=True)  # type: ignore


class Book(ormar.Model):
    class Meta(BaseMeta):
        tablename = "books"
        constraints = [
            ormar.UniqueColumns("source", "remote_id"),
        ]

    id: int = ormar.Integer(primary_key=True, nullable=False)  # type: ignore

    source: Source = ormar.ForeignKey(Source, nullable=False)
    remote_id: int = ormar.Integer(minimum=0, nullable=False)  # type: ignore

    title: str = ormar.String(max_length=256, nullable=False)  # type: ignore
    lang: str = ormar.String(max_length=2, nullable=False)  # type: ignore
    file_type: str = ormar.String(max_length=4, nullable=False)  # type: ignore
    uploaded: date = ormar.Date()  # type: ignore

    authors = ormar.ManyToMany(Author)


class BookAnnotation(ormar.Model):
    class Meta(BaseMeta):
        tablename = "book_annotations"

    id = ormar.Integer(primary_key=True, nullable=False)

    book: Book = ormar.ForeignKey(Book, nullable=False, unique=True)

    title: str = ormar.String(max_length=256, nullable=False, default="")  # type: ignore
    text: str = ormar.Text(nullable=False, default="")  # type: ignore
    file: str = ormar.String(max_length=256, nullable=True)  # type: ignore


class Translation(ormar.Model):
    class Meta(BaseMeta):
        tablename = "translations"
        constraints = [
            ormar.UniqueColumns("book", "translator"),
            ormar.UniqueColumns("book", "position"),
        ]

    id: int = ormar.Integer(primary_key=True, nullable=False)  # type: ignore

    book: Book = ormar.ForeignKey(Book, nullable=False)
    translator: Author = ormar.ForeignKey(Author, nullable=False)
    position: int = ormar.SmallInteger(nullable=False)  # type: ignore


class Sequence(ormar.Model):
    class Meta(BaseMeta):
        tablename = "sequences"
        constraints = [
            ormar.UniqueColumns("source", "remote_id"),
        ]

    id: int = ormar.Integer(primary_key=True, nullable=False)  # type: ignore

    source: Source = ormar.ForeignKey(Source, nullable=False)
    remote_id: int = ormar.Integer(minimum=0, nullable=False)  # type: ignore

    name: str = ormar.String(max_length=256, nullable=False)  # type: ignore


class SequenceInfo(ormar.Model):
    class Meta(BaseMeta):
        tablename = "sequence_infos"
        constraints = [
            ormar.UniqueColumns("book", "sequence"),
            ormar.UniqueColumns("sequence", "position"),
        ]

    id: int = ormar.Integer(primary_key=True, nullable=False)  # type: ignore

    book: Book = ormar.ForeignKey(Book, nullable=False)
    sequence: Sequence = ormar.ForeignKey(Sequence, nullable=False)
    position: int = ormar.SmallInteger(minimum=0, nullable=False)  # type: ignore
