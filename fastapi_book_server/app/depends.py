from fastapi import Security, HTTPException, status

from core.auth import default_security
from core.config import env_config


async def check_token(api_key: str = Security(default_security)):
    if api_key != env_config.API_KEY:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN, detail="Wrong api key!"
        )
