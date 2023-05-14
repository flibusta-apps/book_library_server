from contextlib import asynccontextmanager

from fastapi import FastAPI
from fastapi.responses import ORJSONResponse

from fastapi_pagination import add_pagination
from prometheus_fastapi_instrumentator import Instrumentator
from redis import asyncio as aioredis
import sentry_sdk

from app.views import routers
from core.config import env_config
from core.db import database


sentry_sdk.init(
    env_config.SENTRY_SDN,
)


@asynccontextmanager
async def lifespan(app: FastAPI):
    database_ = app.state.database
    if not database_.is_connected:
        await database_.connect()

    yield

    database_ = app.state.database
    if database_.is_connected:
        await database_.disconnect()

    await app.state.redis.close()


def start_app() -> FastAPI:
    app = FastAPI(default_response_class=ORJSONResponse, lifespan=lifespan)

    app.state.database = database

    app.state.redis = aioredis.Redis(
        host=env_config.REDIS_HOST,
        port=env_config.REDIS_PORT,
        db=env_config.REDIS_DB,
        password=env_config.REDIS_PASSWORD,
    )

    for router in routers:
        app.include_router(router)

    add_pagination(app)

    Instrumentator(
        should_ignore_untemplated=True,
        excluded_handlers=["/docs", "/metrics", "/healthcheck"],
    ).instrument(app).expose(app, include_in_schema=True)

    return app
