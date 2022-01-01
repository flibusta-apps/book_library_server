from typing import Optional


def get_book_filter(is_deleted: Optional[bool] = None) -> dict:
    result = {}

    if is_deleted is not None:
        result["is_deleted"] = is_deleted

    return result
