-- Create book_authors junction table
CREATE TABLE IF NOT EXISTS book_authors (
    id SERIAL PRIMARY KEY,
    author INTEGER NOT NULL,
    book INTEGER NOT NULL,
    CONSTRAINT uc_book_authors_book_author UNIQUE (book, author),
    CONSTRAINT fk_book_authors_authors_author_id FOREIGN KEY (author) REFERENCES authors(id) ON UPDATE CASCADE ON DELETE CASCADE,
    CONSTRAINT fk_book_authors_books_book_id FOREIGN KEY (book) REFERENCES books(id) ON UPDATE CASCADE ON DELETE CASCADE
);

-- Create indexes for book_authors
CREATE INDEX IF NOT EXISTS book_authors_author ON book_authors (author);
CREATE INDEX IF NOT EXISTS book_authors_book ON book_authors (book);
