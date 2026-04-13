use std::hint::spin_loop;
use std::time::SystemTime;
use crate::error::SnowflakeError;

/// Returns the number of milliseconds elapsed since `epoch`.
#[inline]
pub fn get_time_millis(epoch: SystemTime) -> Result<i64, SnowflakeError> {
    SystemTime::now()
        .duration_since(epoch)
        .map(|d| d.as_millis() as i64)
        .map_err(|_| SnowflakeError::ClockBeforeEpoch)
}

/// Busy-waits until the clock advances past `last_millis`.
#[inline]
pub fn biding_time_conditions(
    last_millis: i64,
    epoch: SystemTime,
) -> Result<i64, SnowflakeError> {
    loop {
        let now = get_time_millis(epoch)?;
        if now > last_millis {
            return Ok(now);
        }
        spin_loop();
    }
}
