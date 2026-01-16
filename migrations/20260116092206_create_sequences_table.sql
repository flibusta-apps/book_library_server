-- Create sequences table
CREATE TABLE IF NOT EXISTS sequences (
    id SERIAL PRIMARY KEY,
    source SMALLINT NOT NULL,
    remote_id INTEGER NOT NULL,
    name VARCHAR(256) NOT NULL,
    CONSTRAINT uc_sequences_source_remote_id UNIQUE (source, remote_id),
    CONSTRAINT fk_sequences_sources_id_source FOREIGN KEY (source) REFERENCES sources(id)
);
