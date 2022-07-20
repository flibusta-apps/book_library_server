from datetime import date
from typing import Optional

import ormar
from sqlalchemy import text

from core.db import metadata, database


class BaseMeta(ormar.ModelMeta):
    metadata = metadata
    database = database


class Source(ormar.Model):
    class Meta(BaseMeta):
        tablename = "sources"

    id: int = ormar.SmallInteger(primary_key=True, nullable=False)  # type: ignore
    name: str = ormar.String(max_length=32, nullable=False, unique=True)  # type: ignore


class Genre(ormar.Model):
    class Meta(BaseMeta):
        tablename = "genres"
        constraints = [
            ormar.UniqueColumns("source", "remote_id"),
        ]

    id: int = ormar.Integer(primary_key=True, nullable=False)  # type: ignore

    source: Source = ormar.ForeignKey(Source, nullable=False)
    remote_id: int = ormar.Integer(minimum=0, nullable=False)  # type: ignore

    code: str = ormar.String(max_length=45, nullable=False)  # type: ignore
    description: str = ormar.String(max_length=99, nullable=False)  # type: ignore
    meta: str = ormar.String(max_length=45, nullable=False)  # type: ignore


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
    middle_name: str = ormar.String(max_length=256, nullable=True)  # type: ignore

    @ormar.property_field
    def annotation_exists(self) -> bool:
        return len(self.annotations) != 0


class AuthorAnnotation(ormar.Model):
    class Meta(BaseMeta):
        tablename = "author_annotations"

    id = ormar.Integer(primary_key=True, nullable=False)

    author: Author = ormar.ForeignKey(
        Author, nullable=False, unique=True, related_name="annotations"
    )

    title: str = ormar.String(
        max_length=256, nullable=False, default=""
    )  # type: ignore
    text: str = ormar.Text(nullable=False, default="")  # type: ignore
    file: str = ormar.String(max_length=256, nullable=True)  # type: ignore


class Sequence(ormar.Model):
    class Meta(BaseMeta):
        tablename = "sequences"
        constraints = [
            ormar.UniqueColumns("source", "remote_id"),
        ]

    id: int = ormar.Integer(primary_key=True, nullable=False)  # type: ignore

    source: Source = ormar.ForeignKey(Source, nullable=False)
    remote_id: int = ormar.Integer(minimum=0, nullable=False)  # type: ignore

    name: str = ormar.String(max_length=256, nullable=False, index=True)  # type: ignore


class BookAuthors(ormar.Model):
    class Meta(BaseMeta):
        tablename = "book_authors"

    id: int = ormar.Integer(primary_key=True, nullable=False)  # type: ignore


class BookGenres(ormar.Model):
    class Meta(BaseMeta):
        tablename = "book_genres"

    id: int = ormar.Integer(primary_key=True, nullable=False)  # type: ignore


class BookSequences(ormar.Model):
    class Meta(BaseMeta):
        tablename = "book_sequences"
        orders_by = [
            "position",
        ]

    id: int = ormar.Integer(primary_key=True, nullable=False)  # type: ignore

    position: int = ormar.SmallInteger(minimum=0, nullable=False)  # type: ignore


class Translation(ormar.Model):
    class Meta(BaseMeta):
        tablename = "translations"
        orders_by = [
            "position",
        ]

    id: int = ormar.Integer(primary_key=True, nullable=False)  # type: ignore

    position: int = ormar.SmallInteger(nullable=False)  # type: ignore


class Book(ormar.Model):
    class Meta(BaseMeta):
        tablename = "books"
        constraints = [
            ormar.UniqueColumns("source", "remote_id"),
        ]

    id: int = ormar.Integer(primary_key=True, nullable=False)  # type: ignore

    source: Source = ormar.ForeignKey(Source, nullable=False)
    remote_id: int = ormar.Integer(minimum=0, nullable=False)  # type: ignore

    title: str = ormar.String(
        max_length=256, nullable=False, index=True
    )  # type: ignore
    lang: str = ormar.String(max_length=3, nullable=False, index=True)  # type: ignore
    file_type: str = ormar.String(
        max_length=4, nullable=False, index=True
    )  # type: ignore
    uploaded: date = ormar.Date()  # type: ignore
    is_deleted: bool = ormar.Boolean(
        default=False, server_default=text("false"), nullable=False
    )
    pages: Optional[int] = ormar.Integer(minimum=0, nullable=True)  # type: ignore

    authors = ormar.ManyToMany(Author, through=BookAuthors)
    translators = ormar.ManyToMany(
        Author, through=Translation, related_name="translated_books"
    )
    genres = ormar.ManyToMany(Genre, through=BookGenres)
    sequences = ormar.ManyToMany(Sequence, through=BookSequences)

    @ormar.property_field
    def available_types(self) -> list[str]:
        if self.file_type == "fb2" and self.source.name == "flibusta":
            return ["fb2", "fb2zip", "epub", "mobi"]

        return [self.file_type]

    @ormar.property_field
    def annotation_exists(self) -> bool:
        return len(self.annotations) != 0


class BookAnnotation(ormar.Model):
    class Meta(BaseMeta):
        tablename = "book_annotations"

    id = ormar.Integer(primary_key=True, nullable=False)

    book: Book = ormar.ForeignKey(
        Book, nullable=False, unique=True, related_name="annotations"
    )

    title: str = ormar.String(
        max_length=256, nullable=False, default=""
    )  # type: ignore
    text: str = ormar.Text(nullable=False, default="")  # type: ignore
    file: str = ormar.String(max_length=256, nullable=True)  # type: ignore
