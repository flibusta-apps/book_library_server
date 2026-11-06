# syntax=docker/dockerfile:1
FROM rust:1-bookworm@sha256:77fac8b98f9f46062bb680b6d25d5bcaabfc400143952ebc572e924bcbedc3fa AS builder

WORKDIR /app

ENV SQLX_OFFLINE=true

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --bin book_library_server \
    && cp /app/target/release/book_library_server /app/book_library_server


FROM debian:bookworm-slim@sha256:7b140f374b289a7c2befc338f42ebe6441b7ea838a042bbd5acbfca6ec875818

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
