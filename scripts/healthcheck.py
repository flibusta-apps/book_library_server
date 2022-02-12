import httpx


response = httpx.get(
    "http://localhost:8080/healthcheck",
)
print(f"HEALTHCHECK STATUS: {response.status_code}")
exit(0 if response.status_code == 200 else 1)
