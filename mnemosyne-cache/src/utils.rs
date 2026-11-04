use std::time::Duration;

use redis::{ErrorKind, RedisError, RedisResult};

pub fn ttl_seconds(ttl: Duration) -> RedisResult<u64> {
    let seconds = ttl.as_secs();

    if seconds == 0 {
        return Err(RedisError::from((
            ErrorKind::Client,
            "Cache TTL must be at least one second",
        )));
    }

    Ok(seconds)
}

pub fn codec_error(message: &'static str, error: impl std::fmt::Display) -> RedisError {
    RedisError::from((ErrorKind::Client, message, error.to_string()))
}
