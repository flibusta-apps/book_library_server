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

#[cfg(test)]
mod tests {
    use super::*;

    fn deserialize(value: serde_json::Value) -> Result<Pagination, serde_json::Error> {
        serde_json::from_value(value)
    }

    #[test]
    fn defaults_when_empty() {
        let pagination = deserialize(serde_json::json!({})).expect("should deserialize");
        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.size, 50);
    }

    #[test]
    fn rejects_page_less_than_one() {
        let result = deserialize(serde_json::json!({ "page": 0, "size": 10 }));
        assert!(result.is_err());

        let result = deserialize(serde_json::json!({ "page": -1, "size": 10 }));
        assert!(result.is_err());
    }

    #[test]
    fn rejects_size_less_than_one() {
        let result = deserialize(serde_json::json!({ "page": 1, "size": 0 }));
        assert!(result.is_err());

        let result = deserialize(serde_json::json!({ "page": 1, "size": -5 }));
        assert!(result.is_err());
    }

    #[test]
    fn rejects_size_over_max() {
        let result = deserialize(serde_json::json!({ "page": 1, "size": MAX_PAGE_SIZE + 1 }));
        assert!(result.is_err());
    }

    #[test]
    fn accepts_boundary_values() {
        let pagination =
            deserialize(serde_json::json!({ "page": 1, "size": 1 })).expect("size=1 valid");
        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.size, 1);

        let pagination = deserialize(serde_json::json!({ "page": 1, "size": MAX_PAGE_SIZE }))
            .expect("size=MAX_PAGE_SIZE valid");
        assert_eq!(pagination.size, MAX_PAGE_SIZE);
    }

    #[test]
    fn offset_is_computed_correctly() {
        let pagination = Pagination { page: 1, size: 20 };
        assert_eq!(pagination.offset(), 0);

        let pagination = Pagination { page: 3, size: 20 };
        assert_eq!(pagination.offset(), 40);

        let pagination = Pagination { page: 5, size: 200 };
        assert_eq!(pagination.offset(), 800);
    }

    #[test]
    fn calc_pages_zero_total() {
        assert_eq!(calc_pages(0, 10), 0);
    }

    #[test]
    fn calc_pages_exact_division() {
        assert_eq!(calc_pages(100, 10), 10);
        assert_eq!(calc_pages(20, 20), 1);
    }

    #[test]
    fn calc_pages_with_remainder() {
        assert_eq!(calc_pages(101, 10), 11);
        assert_eq!(calc_pages(1, 10), 1);
        assert_eq!(calc_pages(21, 20), 2);
    }

    #[test]
    fn calc_pages_never_divides_by_zero_because_size_is_validated() {
        // `size` is always >= 1 by the time `calc_pages` is called (enforced
        // by `Pagination`'s `Deserialize` impl), but exercise the smallest
        // valid size directly to make that invariant explicit.
        assert_eq!(calc_pages(5, 1), 5);
        assert_eq!(calc_pages(0, 1), 0);
    }

    #[test]
    fn page_new_populates_all_fields() {
        let pagination = Pagination { page: 2, size: 10 };
        let page: Page<i32> = Page::new(vec![1, 2, 3], 23, &pagination);

        assert_eq!(page.items, vec![1, 2, 3]);
        assert_eq!(page.total, 23);
        assert_eq!(page.page, 2);
        assert_eq!(page.size, 10);
        assert_eq!(page.pages, 3);
    }

    #[test]
    fn page_with_parent_new_populates_all_fields() {
        let pagination = Pagination { page: 1, size: 5 };
        let page: PageWithParent<i32, &str> =
            PageWithParent::new("parent", vec![1, 2], 5, &pagination);

        assert_eq!(page.parent_item, "parent");
        assert_eq!(page.items, vec![1, 2]);
        assert_eq!(page.total, 5);
        assert_eq!(page.page, 1);
        assert_eq!(page.size, 5);
        assert_eq!(page.pages, 1);
    }
}
