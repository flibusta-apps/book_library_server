-- Spec 12.2: replace the `CASE WHEN b.file_type = 'fb2' THEN ... ELSE ... END`
-- expression duplicated verbatim across ~10 queries with a single reusable,
-- IMMUTABLE SQL function so the planner can inline/fold it and a new format
-- only requires touching one migration.
CREATE OR REPLACE FUNCTION available_types(file_type text) RETURNS text[]
    IMMUTABLE
    LANGUAGE sql
AS $$
    SELECT CASE
        WHEN file_type = 'fb2' THEN ARRAY['fb2', 'epub', 'mobi', 'fb2zip']::text[]
        ELSE ARRAY[file_type]::text[]
    END
$$;
