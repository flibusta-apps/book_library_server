use serde::{de, Deserialize, Deserializer, Serialize};

fn default_page() -> i64 {
    1
}

fn default_size() -> i64 {
    50
}

/// Maximum allowed page `size` to prevent accidental (or malicious) requests
/// for huge result sets (see docs/specs/06-pagination-input-validation.md).
pub const MAX_PAGE_SIZE: i64 = 200;

/// Raw, unvalidated shape used only to drive `serde` deserialization; never
/// exposed outside this module.
#[derive(Deserialize)]
struct RawPagination {
    #[serde(default = "default_page")]
    page: i64,
    #[serde(default = "default_size")]
    size: i64,
}

/// Validated pagination parameters.
///
/// This type has a custom `Deserialize` implementation that enforces
/// `page >= 1` and `1 <= size <= MAX_PAGE_SIZE`. Because every handler
/// extracts pagination via `axum::extract::Query<Pagination>`, invalid
/// values are rejected with a `400 Bad Request` by axum's `Query` extractor
/// before any handler code runs -- there is a single, shared validation
/// path for all list endpoints.
pub struct Pagination {
    pub page: i64,
    pub size: i64,
}

impl Pagination {
    /// Row offset for the current page. Safe to use directly in `OFFSET`/
    /// Meilisearch offset since `page >= 1` and `size >= 1` are guaranteed
    /// by validation.
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.size
    }
}

impl<'de> Deserialize<'de> for Pagination {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawPagination::deserialize(deserializer)?;

        if raw.page < 1 {
            return Err(de::Error::custom("`page` must be >= 1"));
        }

        if raw.size < 1 || raw.size > MAX_PAGE_SIZE {
            return Err(de::Error::custom(format!(
                "`size` must be between 1 and {MAX_PAGE_SIZE}"
            )));
        }

        Ok(Pagination {
            page: raw.page,
            size: raw.size,
        })
    }
}

/// Shared "total pages" calculation. Safe from division-by-zero because
/// `Pagination` validation guarantees `size > 0`.
fn calc_pages(total: i64, size: i64) -> i64 {
    let total = total.max(0);
    (total + size - 1) / size
}

#[derive(Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub size: i64,
    pub pages: i64,
}

#[derive(Serialize)]
pub struct PageWithParent<T, P> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub size: i64,
    pub pages: i64,
    pub parent_item: P,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, total: i64, pagination: &Pagination) -> Self {
        Self {
            items,
            total,
            page: pagination.page,
            size: pagination.size,
            pages: calc_pages(total, pagination.size),
        }
    }
}

impl<T, P> PageWithParent<T, P> {
    pub fn new(parent_item: P, items: Vec<T>, total: i64, pagination: &Pagination) -> Self {
        Self {
            items,
            total,
            page: pagination.page,
            size: pagination.size,
            pages: calc_pages(total, pagination.size),
            parent_item,
        }
    }
}
