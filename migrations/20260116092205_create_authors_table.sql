-- Create authors table
CREATE TABLE IF NOT EXISTS authors (
    id SERIAL PRIMARY KEY,
    source SMALLINT NOT NULL,
    remote_id INTEGER NOT NULL,
    first_name VARCHAR(256) NOT NULL,
    last_name VARCHAR(256) NOT NULL,
    middle_name VARCHAR(256),
    CONSTRAINT uc_authors_source_remote_id UNIQUE (source, remote_id),
    CONSTRAINT fk_authors_sources_id_source FOREIGN KEY (source) REFERENCES sources(id)
);

-- Create trigram indexes for author search
CREATE INDEX IF NOT EXISTS tgrm_authors_lf ON authors USING gin ((last_name || ' ' || first_name) gin_trgm_ops);
CREATE INDEX IF NOT EXISTS tgrm_authors_lfm ON authors USING gin ((last_name || ' ' || first_name || ' ' || middle_name) gin_trgm_ops);
