-- Create books table
CREATE TABLE IF NOT EXISTS books (
    id SERIAL PRIMARY KEY,
    source SMALLINT NOT NULL,
    remote_id INTEGER NOT NULL,
    title VARCHAR(256) NOT NULL,
    lang VARCHAR(3) NOT NULL,
    file_type VARCHAR(4) NOT NULL,
    uploaded DATE NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    pages INTEGER,
    year SMALLINT NOT NULL DEFAULT 0,
    CONSTRAINT uc_books_source_remote_id UNIQUE (source, remote_id),
    CONSTRAINT fk_books_sources_id_source FOREIGN KEY (source) REFERENCES sources(id)
);

-- Create indexes for books
CREATE INDEX IF NOT EXISTS idx_id_asc__not_is_deleted ON books (id) WHERE NOT is_deleted;
CREATE INDEX IF NOT EXISTS idx_id_asc__uploaded__not_is_deleted ON books (id, uploaded) WHERE NOT is_deleted;
CREATE INDEX IF NOT EXISTS idx_uploaded__id_asc__not_is_deleted ON books (uploaded, id) WHERE NOT is_deleted;
