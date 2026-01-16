-- Create author_annotations table
CREATE TABLE IF NOT EXISTS author_annotations (
    id SERIAL PRIMARY KEY,
    author INTEGER NOT NULL UNIQUE,
    title VARCHAR(256) NOT NULL,
    text TEXT NOT NULL,
    file VARCHAR(256),
    CONSTRAINT fk_author_annotations_authors_id_author FOREIGN KEY (author) REFERENCES authors(id)
);

-- Create index for author_annotations
CREATE INDEX IF NOT EXISTS author_annotation_author_id ON author_annotations (author);
