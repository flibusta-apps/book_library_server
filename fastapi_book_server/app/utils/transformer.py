from typing import Any, Sequence, TypeVar

from pydantic import BaseModel


T = TypeVar("T", bound=BaseModel)


async def dict_transformer(items: Sequence[T]) -> Sequence[dict[str, Any]]:
    return [item.dict() for item in items]
