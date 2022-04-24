from fastapi import FastAPI
from fastapi.responses import ORJSONResponse

import aioredis
from fastapi_pagination import add_pagination
from prometheus_fastapi_instrumentator import Instrumentator
import sentry_sdk

from app.views import routers
from core.config import env_config
from core.db import database


sentry_sdk.init(
    env_config.SENTRY_SDN,
)


def start_app() -> FastAPI:
    app = FastAPI(default_response_class=ORJSONResponse)

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

    @app.on_event("startup")
    async def startup() -> None:
        database_ = app.state.database
        if not database_.is_connected:
            await database_.connect()

    @app.on_event("shutdown")
    async def shutdown() -> None:
        database_ = app.state.database
        if database_.is_connected:
            await database_.disconnect()

    Instrumentator().instrument(app).expose(app, include_in_schema=True)

    return app
