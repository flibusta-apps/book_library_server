from operator import add
from fastapi import FastAPI
from fastapi_pagination import add_pagination

from core.db import database
from app.views import routers


def start_app() -> FastAPI:
    app = FastAPI()

    app.state.database = database

    for router in routers:
        app.include_router(router)

    add_pagination(app)

    @app.on_event('startup')
    async def startup() -> None:
        database_ = app.state.database
        if not database_.is_connected:
            await database_.connect()

    @app.on_event('shutdown')
    async def shutdown() -> None:
        database_ = app.state.database
        if database_.is_connected:
            await database_.disconnect()

    return app
