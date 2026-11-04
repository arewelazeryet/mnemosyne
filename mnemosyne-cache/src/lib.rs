use std::time::Duration;

use redis::{AsyncTypedCommands, RedisResult, aio::ConnectionManager};
use serde::{Serialize, de::DeserializeOwned};

use crate::utils::{codec_error, ttl_seconds};

pub mod utils;

pub async fn set<T: Serialize>(
    connection: &ConnectionManager,
    key: &str,
    value: &T,
    ttl: Duration,
) -> RedisResult<()> {
    let mut connection = connection.clone();

    let serialized =
        serde_json::to_string(value).map_err(|e| codec_error("Cache serialization failed", e))?;

    let _ = connection
        .set_ex(key, &serialized, ttl_seconds(ttl)?)
        .await?;

    Ok(())
}

pub async fn get_or_refresh<T, TOut, F, FMap, Fut>(
    connection: &ConnectionManager,
    key: &str,
    ttl: Duration,
    refresh: F,
    map: FMap,
) -> RedisResult<TOut>
where
    TOut: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    FMap: FnOnce(T) -> TOut,
    Fut: Future<Output = RedisResult<T>>,
{
    get_or_refresh_with(
        connection,
        key,
        ttl,
        (),
        move |_| refresh(),
        move |v| map(v),
    )
    .await
}

pub async fn get_or_refresh_with<T, TOut, A, F, FMap, Fut>(
    connection: &ConnectionManager,
    key: &str,
    ttl: Duration,
    argument: A,
    refresh: F,
    map: FMap,
) -> RedisResult<TOut>
where
    TOut: Serialize + DeserializeOwned,
    F: FnOnce(A) -> Fut,
    FMap: FnOnce(T) -> TOut,
    Fut: Future<Output = RedisResult<T>>,
{
    let mut connection = connection.clone();

    // Try cache first.
    let cached: Option<String> = connection.get(key).await?;

    if let Some(json) = cached {
        return serde_json::from_str(&json)
            .map_err(|e| codec_error("Cache deserialization failed", e));
    }

    // Cache miss: load from the source.
    let value: T = refresh(argument).await?;

    let value: TOut = map(value);

    // Populate cache.
    let serialized =
        serde_json::to_string(&value).map_err(|e| codec_error("Cache serialization failed", e))?;

    let _: () = connection
        .set_ex(key, serialized, ttl_seconds(ttl)?)
        .await?;

    Ok(value)
}
