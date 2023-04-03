cd /app

rm -rf prometheus
mkdir prometheus

alembic -c ./app/alembic.ini upgrade head
gunicorn -k gunicorn_pushgateway_workers.workers.uvicorn.UvicornWorker main:app --bind 0.0.0.0:8080 --timeout 600
