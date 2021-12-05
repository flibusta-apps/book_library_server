from typing import Protocol, TypeVar, Any, Generic, Sequence, runtime_checkable

from pydantic import PositiveInt

from fastapi_pagination import Page, Params
from fastapi_pagination.bases import AbstractParams


@runtime_checkable
class ToDict(Protocol):
    def dict(self) -> dict:
        ...


T = TypeVar('T', ToDict, Any)


class CustomPage(Page[T], Generic[T]):
    total_pages: PositiveInt  

    @classmethod
    def create(
        cls,
        items: Sequence[T],
        total: int,
        params: AbstractParams,
    ) -> Page[T]:
        if not isinstance(params, Params):
            raise ValueError("Page should be used with Params")

        return cls(
            total=total,
            items=[item.dict() for item in items],
            page=params.page,
            size=params.size,
            total_pages=(total + params.size - 1) // params.size,
        )
