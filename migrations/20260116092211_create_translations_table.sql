-- Create translations junction table
CREATE TABLE IF NOT EXISTS translations (
    id SERIAL PRIMARY KEY,
    position SMALLINT NOT NULL,
    author INTEGER NOT NULL,
    book INTEGER NOT NULL,
    CONSTRAINT uc_translations_book_author UNIQUE (book, author),
    CONSTRAINT fk_translations_authors_author_id FOREIGN KEY (author) REFERENCES authors(id) ON UPDATE CASCADE ON DELETE CASCADE,
    CONSTRAINT fk_translations_books_book_id FOREIGN KEY (book) REFERENCES books(id) ON UPDATE CASCADE ON DELETE CASCADE
);

-- Create indexes for translations
CREATE INDEX IF NOT EXISTS translations_author ON translations (author);
CREATE INDEX IF NOT EXISTS translations_book ON translations (book);
