from fastapi import APIRouter


healtcheck_router = APIRouter(tags=["healthcheck"])


@healtcheck_router.get("/healthcheck")
async def healthcheck():
    return "Ok!"
