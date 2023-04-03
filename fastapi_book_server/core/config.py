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

    MEILI_HOST: str
    MEILI_MASTER_KEY: str

    PUSH_GETAWAY_ENABLED: bool = True
    PUSH_GETAWAY_HOST: str = ""
    PUSH_GETAWAY_JOB: str = "library_server"
    PUSH_GETAWAY_INTERVAL: int = 15

    SENTRY_SDN: str

    class Config:
        env_file = ".env"
        env_file_encoding = "utf-8"


env_config = EnvConfig()
