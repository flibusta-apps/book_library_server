from typing import Optional

from fastapi import Security, HTTPException, Query, status

from core.auth import default_security
from core.config import env_config


def check_token(api_key: str = Security(default_security)):
    if api_key != env_config.API_KEY:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN, detail="Wrong api key!"
        )


def get_allowed_langs(
    allowed_langs: Optional[list[str]] = Query(None),
) -> frozenset[str]:
    if allowed_langs is not None:
        return frozenset(allowed_langs)

    return frozenset(("ru", "be", "uk"))
