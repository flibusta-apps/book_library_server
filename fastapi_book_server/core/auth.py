from fastapi.security import APIKeyHeader


default_security = APIKeyHeader(name="Authorization")
