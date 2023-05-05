from math import ceil
from typing import (
    Any,
    Generic,
    Protocol,
    Sequence,
    TypeVar,
    runtime_checkable,
)

from fastapi_pagination import Params
from fastapi_pagination.bases import AbstractParams, BasePage
from fastapi_pagination.types import GreaterEqualOne, GreaterEqualZero
import orjson
from app.utils.orjson_default import orjson_dumps


@runtime_checkable
class ToDict(Protocol):
    def dict(self) -> dict:
        ...


T = TypeVar("T", ToDict, Any)


class Page(BasePage[T], Generic[T]):
    page: GreaterEqualOne
    size: GreaterEqualOne
    total_pages: GreaterEqualZero

    __params_type__ = Params

    class Config:
        json_loads = orjson.loads
        json_dumps = orjson_dumps

    @classmethod
    def create(
        cls,
        items: Sequence[T],
        params: AbstractParams,
        *,
        total: int,
        **kwargs: Any,
    ) -> "Page[T]":
        if not isinstance(params, Params):
            raise ValueError("Page should be used with Params")

        pages = ceil(total / params.size)

        return cls(
            total=total,
            items=items,
            page=params.page,
            size=params.size,
            total_pages=pages,
            **kwargs,
        )
