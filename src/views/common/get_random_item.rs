use meilisearch_sdk::indexes::Index;
use rand::Rng;
use serde::de::DeserializeOwned;

use crate::{error::ApiError, meilisearch::GetId};

/// Meilisearch's default `maxTotalHits` setting, which caps how far
/// offset+limit pagination can reach into an index. Keep this in sync
/// if that setting is ever changed on the Meilisearch instance.
const MEILI_MAX_TOTAL_HITS: usize = 1000;

pub async fn get_random_item<T>(index: Index, filter: String) -> Result<i32, ApiError>
where
    T: DeserializeOwned + GetId + 'static + Send + Sync,
{
    let result = index.search().with_filter(&filter).execute::<T>().await?;

    let total_hits = result.estimated_total_hits.unwrap_or(0);
    if total_hits == 0 {
        return Err(ApiError::NotFound);
    }

    let offset: usize = rand::rng().random_range(0..total_hits.min(MEILI_MAX_TOTAL_HITS));

    let result = index
        .search()
        .with_filter(&filter)
        .with_limit(1)
        .with_offset(offset)
        .execute::<T>()
        .await?;

    let item = result.hits.first().ok_or(ApiError::NotFound)?;

    Ok(item.result.get_id())
}
