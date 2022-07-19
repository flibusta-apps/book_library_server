from typing import Optional


def get_genre_filter(meta: Optional[str] = None) -> dict:
    result = {}

    if meta:
        result["meta"] = meta

    return result
