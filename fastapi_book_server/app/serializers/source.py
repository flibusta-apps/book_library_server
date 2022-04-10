from pydantic import BaseModel


class Source(BaseModel):
    id: int
    name: str
