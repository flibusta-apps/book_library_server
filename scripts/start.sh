cd /app
alembic -c ./app/alembic.ini upgrade head
uvicorn main:app --host="0.0.0.0" --port="8080"
