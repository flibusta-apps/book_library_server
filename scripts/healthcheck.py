import os

import httpx


CHECK_URL = os.environ.get("HEALTHCHECK_URL", "http://localhost:8080/healthcheck")

response = httpx.get(CHECK_URL)

print(f"HEALTHCHECK STATUS: {response.text}")
exit(0 if response.status_code == 200 else 1)
