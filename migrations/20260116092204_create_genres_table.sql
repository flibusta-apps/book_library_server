-- Create genres table
CREATE TABLE IF NOT EXISTS genres (
    id SERIAL PRIMARY KEY,
    source SMALLINT NOT NULL,
    remote_id INTEGER NOT NULL,
    code VARCHAR(45) NOT NULL,
    description VARCHAR(99) NOT NULL,
    meta VARCHAR(45) NOT NULL,
    CONSTRAINT uc_genres_source_remote_id UNIQUE (source, remote_id),
    CONSTRAINT fk_genres_sources_id_source FOREIGN KEY (source) REFERENCES sources(id)
);
