-- Add indexes on books.lang to speed up WHERE lang = ANY($1) filters
CREATE INDEX IF NOT EXISTS idx_books_lang_id ON books (lang, id);
CREATE INDEX IF NOT EXISTS idx_books_lang_id_not_deleted ON books (lang, id) WHERE NOT is_deleted;
