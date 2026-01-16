-- Create sources table
CREATE TABLE IF NOT EXISTS sources (
    id SMALLSERIAL PRIMARY KEY,
    name VARCHAR(32) NOT NULL UNIQUE
);
