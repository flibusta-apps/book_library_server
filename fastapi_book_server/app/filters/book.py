from typing import Optional

from fastapi.params import Query

from app.depends import get_allowed_langs


def get_book_filter(
    is_deleted: Optional[bool] = None, allowed_langs: Optional[list[str]] = Query(None)
) -> dict:
    result = {}

    if is_deleted is not None:
        result["is_deleted"] = is_deleted

    if not (allowed_langs and "__all__" in allowed_langs):
        result["lang__in"] = get_allowed_langs(allowed_langs)

    return result
