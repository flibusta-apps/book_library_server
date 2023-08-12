use meilisearch_sdk::indexes::Index;
use rand::Rng;
use serde::de::DeserializeOwned;

use crate::meilisearch::GetId;

pub async fn get_random_item<'a, T>(
    index: Index, filter: String,
) -> i32
where T: DeserializeOwned + GetId + 'static {
    let result = index
        .search()
        .with_filter(&filter)
        .execute::<T>()
        .await
        .unwrap();

    let offset: usize = rand::thread_rng().gen_range(0..result.estimated_total_hits.unwrap().try_into().unwrap());

    let result = index
        .search()
        .with_limit(1)
        .with_offset(offset)
        .execute::<T>()
        .await
        .unwrap();

    let item = &result.hits.get(0).unwrap().result;

    item.get_id()
}
