use serde::{Deserialize, Serialize};


fn default_page() -> i64 {
    1
}

fn default_size() -> i64 {
    50
}

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_size")]
    pub size: i64
}


#[derive(Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub size: i64,
    pub pages: i64
}

#[derive(Serialize)]
pub struct PageWithParent<T, P> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub size: i64,
    pub pages: i64,
    pub parent_item: P
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, total: i64, pagination: &Pagination) -> Self {
        Self {
            items,
            total,
            page: pagination.page,
            size: pagination.size,
            pages: (total + pagination.size - 1) / pagination.size
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
            pages: (total + pagination.size - 1) / pagination.size,
            parent_item
        }
    }
}
