from urllib.parse import quote

from databases import Database
from sqlalchemy import MetaData

from core.config import env_config

DATABASE_URL = (
    f"postgresql://{env_config.POSTGRES_USER}:{quote(env_config.POSTGRES_PASSWORD)}@"
    f"{env_config.POSTGRES_HOST}:{env_config.POSTGRES_PORT}/{env_config.POSTGRES_DB}"
)

metadata = MetaData()
database = Database(DATABASE_URL)
