from fastapi import APIRouter, Depends

from app.depends import check_token


healtcheck_router = APIRouter(
    prefix="/api/v1",
    tags=["healthcheck"],
    dependencies=[Depends(check_token)],
)


@healtcheck_router.get("/healthcheck")
async def healthcheck():
    return "Ok!"
