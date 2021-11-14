from pydantic import BaseModel


class Sequence(BaseModel):
    id: int
    name: str


class CreateSequence(BaseModel):
    source: int
    remote_id: int
    name: str
