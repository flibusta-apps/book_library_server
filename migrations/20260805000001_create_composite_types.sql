-- Composite types used by `ROW(...)::author_type` / `ROW(...)::sequence_type`
-- casts in aggregate subqueries across src/views/*.rs. These were previously
-- created out-of-band directly on the production database and were missing
-- from the tracked migrations, which meant a fresh database (e.g. a local or
-- CI test database created purely from `migrations/`) could not run any of
-- the composite-returning queries. See docs/specs/10-ci-docker-tests.md (10.2).
-- CREATE TYPE has no IF NOT EXISTS; these types may already exist out-of-band
-- (see note above), so guard each with a duplicate_object catch to keep the
-- migration idempotent across environments where they do/don't exist yet.
DO $$ BEGIN
    CREATE TYPE author_type AS (
        id INTEGER,
        first_name VARCHAR(256),
        last_name VARCHAR(256),
        middle_name VARCHAR(256),
        annotation_exists BOOLEAN
    );
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE sequence_type AS (
        id INTEGER,
        name VARCHAR(256)
    );
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE source_type AS (
        id INTEGER,
        name VARCHAR(256)
    );
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE genre_type AS (
        id INTEGER,
        source source_type,
        remote_id INTEGER,
        code VARCHAR(45),
        description VARCHAR(99),
        meta VARCHAR(45)
    );
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;
