use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::SnowflakeError;
use crate::layout::BitLayout;
use crate::generator::SnowflakeIdGenerator;

/// A buffered wrapper around [`SnowflakeIdGenerator`].
///
/// # Example
///
/// ```rust
/// use snowflake_gen::SnowflakeIdBucket;
///
/// let mut bucket = SnowflakeIdBucket::new(1, 1).unwrap();
/// let id = bucket.get_id();
/// assert!(id > 0);
/// ```
#[derive(Clone, Debug)]
pub struct SnowflakeIdBucket {
    generator: SnowflakeIdGenerator,
    buffer: Vec<i64>,
}

impl SnowflakeIdBucket {
    /// Constructs a bucket using the UNIX epoch and default [`BitLayout`].
    pub fn new(machine_id: i32, node_id: i32) -> Result<Self, SnowflakeError> {
        Self::with_epoch(machine_id, node_id, UNIX_EPOCH)
    }

    /// Constructs a bucket using a custom epoch and default [`BitLayout`].
    pub fn with_epoch(
        machine_id: i32,
        node_id: i32,
        epoch: SystemTime,
    ) -> Result<Self, SnowflakeError> {
        let generator = SnowflakeIdGenerator::with_epoch(machine_id, node_id, epoch)?;
        Ok(Self {
            generator,
            buffer: Vec::new(),
        })
    }

    /// Constructs a bucket with a fully custom [`BitLayout`] and epoch.
    pub fn with_layout_and_epoch(
        machine_id: i32,
        node_id: i32,
        layout: BitLayout,
        epoch: SystemTime,
    ) -> Result<Self, SnowflakeError> {
        let generator =
            SnowflakeIdGenerator::with_layout_and_epoch(machine_id, node_id, layout, epoch)?;
        Ok(Self {
            generator,
            buffer: Vec::new(),
        })
    }

    /// Returns the next ID from the buffer, refilling it when empty.
    pub fn get_id(&mut self) -> i64 {
        if self.buffer.is_empty() {
            self.refill();
        }
        self.buffer.pop().unwrap()
    }

    /// Returns a reference to the underlying [`SnowflakeIdGenerator`].
    #[inline]
    pub fn generator(&self) -> &SnowflakeIdGenerator {
        &self.generator
    }

    fn refill(&mut self) {
        let max_seq = self.generator.layout().max_sequence() as usize;
        self.buffer.reserve(max_seq + 1);
        for _ in 0..=max_seq {
            self.buffer.push(self.generator.lazy_generate());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::{Duration, UNIX_EPOCH};
    use crate::layout::BitLayout;

    #[test]
    fn bucket_ids_are_unique() {
        let mut bucket = SnowflakeIdBucket::new(1, 1).unwrap();
        let mut ids: Vec<i64> = (0..4096).map(|_| bucket.get_id()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 4096);
    }

    #[test]
    fn bucket_multi_refill_unique() {
        let mut bucket = SnowflakeIdBucket::new(1, 1).unwrap();
        // 3 full refills worth of IDs (default max_sequence = 4095, so 4096 per refill)
        let count = 4096 * 3;
        let mut ids: Vec<i64> = (0..count).map(|_| bucket.get_id()).collect();
        let len = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), len, "IDs must stay unique across refills");
    }

    #[test]
    fn bucket_with_epoch() {
        let epoch = UNIX_EPOCH + Duration::from_millis(1_420_070_400_000);
        let mut bucket = SnowflakeIdBucket::with_epoch(1, 1, epoch).unwrap();
        let id = bucket.get_id();
        assert!(id > 0);
    }

    #[test]
    fn bucket_with_layout_and_epoch() {
        let layout = BitLayout::new(38, 8, 7, 10).unwrap();
        // Use a recent epoch so the timestamp fits in 38 bits
        let epoch = UNIX_EPOCH + Duration::from_millis(1_700_000_000_000);
        let mut bucket =
            SnowflakeIdBucket::with_layout_and_epoch(1, 1, layout, epoch).unwrap();
        let id = bucket.get_id();
        assert!(id > 0);
    }
}
