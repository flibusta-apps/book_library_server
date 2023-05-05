from typing import Any

import orjson


def default(value: Any):
    if isinstance(value, frozenset):
        list_value = list(value)
        return "-".join(sorted(list_value))

    return value


def orjson_dumps(v, *, default) -> str:
    return orjson.dumps(v, default=default).decode()
