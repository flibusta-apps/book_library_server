from typing import Optional

from pydantic import BaseSettings


class EnvConfig(BaseSettings):
    API_KEY: str

    POSTGRES_USER: str
    POSTGRES_PASSWORD: str
    POSTGRES_HOST: str
    POSTGRES_PORT: int
    POSTGRES_DB: str

    REDIS_HOST: str
    REDIS_PORT: int
    REDIS_DB: int
    REDIS_PASSWORD: Optional[str]

    class Config:
        env_file = ".env"
        env_file_encoding = "utf-8"


env_config = EnvConfig()
