-- Create book_annotations table
CREATE TABLE IF NOT EXISTS book_annotations (
    id SERIAL PRIMARY KEY,
    book INTEGER NOT NULL UNIQUE,
    title VARCHAR(256) NOT NULL,
    text TEXT NOT NULL,
    file VARCHAR(256),
    CONSTRAINT fk_book_annotations_books_id_book FOREIGN KEY (book) REFERENCES books(id)
);

-- Create index for book_annotations
CREATE INDEX IF NOT EXISTS book_annotation_book_id ON book_annotations (book);
