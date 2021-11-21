from pydantic import BaseModel

from app.serializers.orjson_config import ORJSONConfig


class Source(BaseModel):
    id: int
    name: str

    class Config(ORJSONConfig):
        pass


class CreateSource(BaseModel):
    name: str

    class Config(ORJSONConfig):
        pass
