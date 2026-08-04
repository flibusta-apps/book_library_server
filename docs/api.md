# API contract

Base path: `/api/v1`. All routes below (except `/health` and `/metrics`) require an
`Authorization` header matching `API_KEY` (constant-time compared); unauthenticated
requests get `401`.

Resources: `authors`, `translators`, `genres`, `books`, `sequences` — mounted at
`/api/v1/authors`, `/api/v1/translators`, `/api/v1/genres`, `/api/v1/books`,
`/api/v1/sequences` respectively.

## Pagination envelope

List/search endpoints return:

```json
{
  "items": [...],
  "total": 0,
  "page": 1,
  "size": 50,
  "pages": 0
}
```

`page`/`size` accept `?page=` and `?size=` query params (default `page=1`, `size=50`).
`pages = ceil(total / size)`.

Detail-with-parent endpoints (e.g. `/authors/{id}/books`) return the same shape plus
a `parent_item` field holding the parent resource.

## Search path-segment encoding contract

Search endpoints take the query as a path segment, e.g.:

```
GET /api/v1/books/search/{query}
GET /api/v1/authors/search/{query}
GET /api/v1/sequences/search/{query}
GET /api/v1/translators/search/{query}
```

The server extracts `query` via axum's `Path` extractor, which **only
percent-decodes** the segment (`%XX` → byte). It does **not** apply
`application/x-www-form-urlencoded` semantics — a literal `+` in the path segment
is **not** translated to a space; it stays a literal `+` character.

**Contract:** clients must percent-encode spaces as `%20` (e.g. via
`Url::path_segments_mut().extend(...)` or `percent_encoding::utf8_percent_encode`
with a path-segment-safe encode set), not form-encode them as `+`. A literal `+`
in a user's query (e.g. `C++`) must itself be percent-encoded (`%2B`) by the client
so it survives as a real `+` on the server side.

Example: `война и мир` must be sent as `война%20и%20мир`, not `война+и+мир`.

### Query validation

Before the query is sent to Meilisearch, the server:
1. Trims leading/trailing whitespace.
2. If the trimmed query is empty, returns `200` with an empty page
   (`total: 0`, `items: []`) **without** calling Meilisearch.
3. Truncates the trimmed query to 256 characters (silently — no error).

## 200-vs-404 contract

- **Search endpoints** (`/{resource}/search/{query}`) never return `404`. No
  matches (or an empty/whitespace-only query) → `200` with an empty page
  (`total: 0`, `items: []`).
- **Detail endpoints** (e.g. `GET /books/{id}`, `GET /authors/{id}`,
  `GET /sequences/{id}`, `GET /authors/{id}/annotation`) return `404` when the
  entity does not exist.
- **Random endpoints** (`/books/random`, `/authors/random`, `/sequences/random`)
  return `404` when nothing matches the filter, never a panic/500.

This asymmetry is intentional and stable: consumers (e.g. book_bot) rely on
`total == 0` for "no search results" and on HTTP `404` for "entity not found".
</content>
