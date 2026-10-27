use meilisearch_sdk::indexes::Index;
use rand::Rng;
use serde::de::DeserializeOwned;

use crate::{
    error::ApiError,
    meilisearch::{GetId, MEILI_MAX_TOTAL_HITS, MEILI_TIMEOUT},
};

pub async fn get_random_item<T>(index: Index, filter: String) -> Result<i32, ApiError>
where
    T: DeserializeOwned + GetId + 'static + Send + Sync,
{
    let result = tokio::time::timeout(
        *MEILI_TIMEOUT,
        index
            .search()
            .with_filter(&filter)
            .with_limit(1)
            .execute::<T>(),
    )
    .await
    .map_err(|_| ApiError::MeiliTimeout)??;

    let total_hits = result.estimated_total_hits.unwrap_or(0);
    if total_hits == 0 {
        return Err(ApiError::NotFound);
    }

    // Only the first `MEILI_MAX_TOTAL_HITS` hits are reachable via offset, so
    // pick uniformly among those (documented limitation: not uniform over the
    // whole index when it holds more matches than that).
    let reachable = total_hits.min(MEILI_MAX_TOTAL_HITS);
    let offset: usize = rand::rng().random_range(0..reachable);

    let result = tokio::time::timeout(
        *MEILI_TIMEOUT,
        index
            .search()
            .with_filter(&filter)
            .with_limit(1)
            .with_offset(offset)
            .execute::<T>(),
    )
    .await
    .map_err(|_| ApiError::MeiliTimeout)??;

    let item = result.hits.first().ok_or(ApiError::NotFound)?;

    Ok(item.result.get_id())
}
