====================
book_library_server
====================

A Rust web server (built with Axum and sqlx) that serves the book library
API: it reads book/author/translator/sequence/genre metadata from
PostgreSQL and delegates random-pick and search queries to Meilisearch.

Environment Variables
======================

The following environment variables are read by ``Config::load()`` at
startup:

- ``API_KEY`` — auth key required on the ``Authorization`` header for
  protected endpoints.
- ``POSTGRES_USER`` — PostgreSQL user.
- ``POSTGRES_PASSWORD`` — PostgreSQL password.
- ``POSTGRES_HOST`` — PostgreSQL host.
- ``POSTGRES_PORT`` — PostgreSQL port (``u32``).
- ``POSTGRES_DB`` — PostgreSQL database name.
- ``POSTGRES_MAX_CONNECTIONS`` — optional; max size of the PostgreSQL
  connection pool. Default: ``10``.
- ``POSTGRES_ACQUIRE_TIMEOUT_SECS`` — optional; how long to wait for a
  free connection before failing. Default: ``5``.
- ``POSTGRES_STATEMENT_TIMEOUT_SECS`` — optional; ``statement_timeout``
  set on every new connection. Default: ``30``.
- ``MEILI_HOST`` — Meilisearch host URL.
- ``MEILI_MASTER_KEY`` — Meilisearch master/API key.
- ``MEILI_HTTP_TIMEOUT_SECS`` — optional; HTTP timeout for the
  Meilisearch client. Default: ``10``.
- ``SENTRY_DSN`` — optional Sentry project DSN. When unset/empty, Sentry
  is not initialized.

Auth
====

Protected routes require an ``Authorization`` header whose value must
match ``API_KEY`` exactly (there is no ``Bearer`` prefix), checked with a
constant-time comparison to avoid timing side channels. The auth
middleware is applied to all ``/api/v1/*`` routes and to ``/metrics``.
``/health`` and ``/ready`` are not auth-protected.

Routes
======

Health/metrics:

- ``GET /health`` — liveness probe; always ``200`` if the process is up.
  No auth required.
- ``GET /ready`` — readiness probe; ``200`` if PostgreSQL is reachable,
  ``503`` otherwise. No auth required.
- ``GET /metrics`` — Prometheus metrics endpoint. Requires auth.

Books (``/api/v1/books``):

- ``GET /api/v1/books`` — paginated list of books.
- ``GET /api/v1/books/base`` — paginated list of books, minimal fields.
- ``GET /api/v1/books/random`` — a random book.
- ``GET /api/v1/books/remote/{source_id}/{remote_id}`` — a book by
  source + remote id.
- ``GET /api/v1/books/search/{query}`` — full-text search over books.
- ``GET /api/v1/books/{book_id}`` — a book by id.
- ``GET /api/v1/books/{book_id}/annotation`` — a book's annotation.

Authors (``/api/v1/authors``):

- ``GET /api/v1/authors`` — paginated list of authors.
- ``GET /api/v1/authors/random`` — a random author.
- ``GET /api/v1/authors/{author_id}`` — an author by id.
- ``GET /api/v1/authors/{author_id}/annotation`` — an author's
  annotation.
- ``GET /api/v1/authors/{author_id}/books`` — paginated list of an
  author's books.
- ``GET /api/v1/authors/{author_id}/available_types`` — available file
  types for an author's books.
- ``GET /api/v1/authors/search/{query}`` — full-text search over
  authors.

Translators (``/api/v1/translators``):

- ``GET /api/v1/translators/{translator_id}/books`` — paginated list of
  a translator's books.
- ``GET /api/v1/translators/{translator_id}/available_types`` —
  available file types for a translator's books.
- ``GET /api/v1/translators/search/{query}`` — full-text search over
  translators.

Sequences (``/api/v1/sequences``):

- ``GET /api/v1/sequences/random`` — a random sequence.
- ``GET /api/v1/sequences/search/{query}`` — full-text search over
  sequences.
- ``GET /api/v1/sequences/{sequence_id}`` — a sequence by id.
- ``GET /api/v1/sequences/{sequence_id}/available_types`` — available
  file types for a sequence's books.
- ``GET /api/v1/sequences/{sequence_id}/books`` — paginated list of a
  sequence's books.

Genres (``/api/v1/genres``):

- ``GET /api/v1/genres`` — paginated list of genres.
- ``GET /api/v1/genres/metas`` — distinct genre "meta" categories.

Development
===========

.. code-block:: bash

    cargo fmt                  # format (required, pre-commit enforced)
    cargo clippy                # lint (required, pre-commit + CI enforced)
    cargo test                  # unit + integration tests
