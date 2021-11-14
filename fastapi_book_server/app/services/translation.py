from typing import Union

from fastapi import HTTPException, status

from app.serializers.translation import CreateTranslation, CreateRemoteTranslation

from app.models import Translation as TranslationDB, Source as SourceDB, Book as BookDB, Author as AuthorDB


class TranslationCreator:
    @classmethod
    def _raise_bad_request(cls):
        raise HTTPException(status.HTTP_404_NOT_FOUND)

    @classmethod
    async def _create_translation(cls, data: CreateTranslation) -> TranslationDB:
        return await TranslationDB.objects.create(
            **data.dict()
        )

    @classmethod
    async def _create_remote_translation(cls, data: CreateRemoteTranslation) -> TranslationDB:
        source = await SourceDB.objects.get_or_none(id=data.source)

        if source is None:
            cls._raise_bad_request()

        book = await BookDB.objects.get_or_none(source__id=source.id, remote_id=data.remote_book)

        if book is None:
            cls._raise_bad_request()

        translator = await AuthorDB.objects.get_or_none(source__id=source.id, remote_id=data.remote_translator)

        if translator is None:
            cls._raise_bad_request()

        return await TranslationDB.objects.create(
            book=book.id,
            translator=translator.id,
            position=data.position,
        )

    @classmethod
    async def create(cls, data: Union[CreateTranslation, CreateRemoteTranslation]) -> TranslationDB:
        if isinstance(data, CreateTranslation):
            return await cls._create_translation(data)
        if isinstance(data, CreateRemoteTranslation):
            return await cls._create_remote_translation(data)
