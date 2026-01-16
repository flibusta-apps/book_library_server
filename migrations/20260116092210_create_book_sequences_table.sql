-- Create book_sequences junction table
CREATE TABLE IF NOT EXISTS book_sequences (
    id SERIAL PRIMARY KEY,
    position SMALLINT NOT NULL,
    sequence INTEGER NOT NULL,
    book INTEGER NOT NULL,
    CONSTRAINT uc_book_sequences_book_sequence UNIQUE (book, sequence),
    CONSTRAINT fk_book_sequences_sequences_sequence_id FOREIGN KEY (sequence) REFERENCES sequences(id) ON UPDATE CASCADE ON DELETE CASCADE,
    CONSTRAINT fk_book_sequences_books_book_id FOREIGN KEY (book) REFERENCES books(id) ON UPDATE CASCADE ON DELETE CASCADE
);

-- Create indexes for book_sequences
CREATE INDEX IF NOT EXISTS book_sequences_sequence ON book_sequences (sequence);
CREATE INDEX IF NOT EXISTS book_sequences_book ON book_sequences (book);
