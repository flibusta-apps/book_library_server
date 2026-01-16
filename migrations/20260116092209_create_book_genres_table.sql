-- Create book_genres junction table
CREATE TABLE IF NOT EXISTS book_genres (
    id SERIAL PRIMARY KEY,
    genre INTEGER NOT NULL,
    book INTEGER NOT NULL,
    CONSTRAINT uc_book_genres_book_genre UNIQUE (book, genre),
    CONSTRAINT fk_book_genres_genres_genre_id FOREIGN KEY (genre) REFERENCES genres(id) ON UPDATE CASCADE ON DELETE CASCADE,
    CONSTRAINT fk_book_genres_books_book_id FOREIGN KEY (book) REFERENCES books(id) ON UPDATE CASCADE ON DELETE CASCADE
);

-- Create indexes for book_genres
CREATE INDEX IF NOT EXISTS book_genres_genre ON book_genres (genre);
CREATE INDEX IF NOT EXISTS book_genres_book ON book_genres (book);
