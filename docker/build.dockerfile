# syntax=docker/dockerfile:1
FROM rust:bookworm AS builder

WORKDIR /app

ENV SQLX_OFFLINE=true

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --bin book_library_server \
    && cp /app/target/release/book_library_server /app/book_library_server


FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y openssl ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

RUN useradd -m -u 10001 app

WORKDIR /app

COPY --from=builder /app/book_library_server /usr/local/bin/book_library_server

USER app

HEALTHCHECK CMD curl -f http://localhost:8080/health || exit 1

CMD ["/usr/local/bin/book_library_server"]
