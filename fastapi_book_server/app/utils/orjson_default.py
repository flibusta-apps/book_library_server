from typing import Any


def default(value: Any):
    if isinstance(value, frozenset):
        list_value = list(value)
        return "-".join(sorted(list_value))

    return value
