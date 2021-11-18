from typing import Union

from fastapi import HTTPException, status

from app.models import Sequence as SequenceInfoDB, Source as SourceDB, Book as BookDB, Sequence as SequenceDB
from app.serializers.sequence_info import CreateSequenceInfo, CreateRemoteSequenceInfo


class SequenceInfoCreator:
    @classmethod
    def _raise_bad_request(cls):
        raise HTTPException(status.HTTP_404_NOT_FOUND)

    @classmethod
    async def _create_sequence_info(cls, data: CreateSequenceInfo) -> SequenceInfoDB:
        return await SequenceInfoDB.objects.create(**data.dict())

    @classmethod
    async def _create_remote_sequence_info(cls, data: CreateRemoteSequenceInfo) -> SequenceInfoDB:
        source = await SourceDB.objects.get_or_none(id=data.source)

        if source is None:
            cls._raise_bad_request()

        book = await BookDB.objects.get_or_none(source__id=source.id, remote_id=data.remote_book)

        if book is None:
            cls._raise_bad_request()

        sequence = await SequenceDB.objects.get_or_none(source__id=source.id, remote_id=data.remote_sequence)

        if sequence is None:
            cls._raise_bad_request()

        return await SequenceInfoDB.objects.create(
            book=book.id,
            sequence=sequence.id,
            position=data.position,
        )

    @classmethod
    async def create(cls, data: Union[CreateSequenceInfo, CreateRemoteSequenceInfo]) -> SequenceInfoDB:
        if isinstance(data, CreateSequenceInfo):
            return await cls._create_sequence_info(data)
        if isinstance(data, CreateRemoteSequenceInfo):
            return await cls._create_remote_sequence_info(data)
