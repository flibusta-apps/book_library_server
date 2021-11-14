from pydantic import BaseModel


class Source(BaseModel):
    id: int
    name: str


class CreateSource(BaseModel):
    name: str
