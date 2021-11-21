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
