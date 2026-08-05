-- Composite types used by `ROW(...)::author_type` / `ROW(...)::sequence_type`
-- casts in aggregate subqueries across src/views/*.rs. These were previously
-- created out-of-band directly on the production database and were missing
-- from the tracked migrations, which meant a fresh database (e.g. a local or
-- CI test database created purely from `migrations/`) could not run any of
-- the composite-returning queries. See docs/specs/10-ci-docker-tests.md (10.2).
CREATE TYPE author_type AS (
    id INTEGER,
    first_name VARCHAR(256),
    last_name VARCHAR(256),
    middle_name VARCHAR(256),
    annotation_exists BOOLEAN
);

CREATE TYPE sequence_type AS (
    id INTEGER,
    name VARCHAR(256)
);

CREATE TYPE source_type AS (
    id INTEGER,
    name VARCHAR(256)
);

CREATE TYPE genre_type AS (
    id INTEGER,
    source source_type,
    remote_id INTEGER,
    code VARCHAR(45),
    description VARCHAR(99),
    meta VARCHAR(45)
);
