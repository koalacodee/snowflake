use crate::error::SnowflakeError;
use crate::utils::{get_time_millis, biding_time_conditions};
use crate::generator::SnowflakeIdGenerator;

impl SnowflakeIdGenerator {
    /// Generates the next ID, **blocking** until the next millisecond if the
    /// sequence counter wraps.
    ///
    /// ```rust
    /// use snowflake_gen::SnowflakeIdGenerator;
    ///
    /// let mut idgen = SnowflakeIdGenerator::new(1, 1).unwrap();
    /// let id = idgen.generate().unwrap();
    /// assert!(id > 0);
    /// ```
    #[must_use = "generated ID is discarded; this advances the sequence counter for nothing"]
    pub fn generate(&mut self) -> Result<i64, SnowflakeError> {
        let id = self.assemble(self.last_time_millis);
        self.idx = (self.idx + 1) & self.layout.max_sequence();

        if self.idx == 0 {
            let mut now = get_time_millis(self.epoch)?;
            if now < self.last_time_millis {
                return Err(SnowflakeError::ClockMovedBackwards);
            }
            if now == self.last_time_millis {
                now = biding_time_conditions(self.last_time_millis, self.epoch)?;
            }
            self.last_time_millis = now;
        }

        Ok(id)
    }

    /// Generates the next ID, **always** reading the current clock.
    #[must_use = "generated ID is discarded; this advances the sequence counter for nothing"]
    pub fn real_time_generate(&mut self) -> Result<i64, SnowflakeError> {
        let mut now = get_time_millis(self.epoch)?;

        if now < self.last_time_millis {
            return Err(SnowflakeError::ClockMovedBackwards);
        }

        if now != self.last_time_millis {
            self.last_time_millis = now;
            self.idx = 0;
        }

        let id = self.assemble(self.last_time_millis);
        self.idx = (self.idx + 1) & self.layout.max_sequence();

        if self.idx == 0 {
            now = biding_time_conditions(self.last_time_millis, self.epoch)?;
            self.last_time_millis = now;
        }

        Ok(id)
    }

    /// Generates the next ID without reading the system clock.
    ///
    /// Instead of a syscall, `lazy_generate` synthetically increments
    /// `last_time_millis` whenever the sequence counter wraps.  This makes it
    /// the fastest generation mode but comes with an important constraint:
    ///
    /// **Do not mix `lazy_generate` with [`generate`](Self::generate) or
    /// [`real_time_generate`](Self::real_time_generate) on the same generator
    /// instance.**  Because `lazy_generate` can advance `last_time_millis`
    /// ahead of the real clock, a subsequent clock-based call may produce a
    /// timestamp that `lazy_generate` already used, resulting in **duplicate
    /// IDs**.
    ///
    /// `SnowflakeIdBucket` uses `lazy_generate` internally on a dedicated
    /// generator that is never exposed for clock-based calls, so it is safe.
    #[must_use = "generated ID is discarded; this advances the sequence counter for nothing"]
    pub fn lazy_generate(&mut self) -> i64 {
        let id = self.assemble(self.last_time_millis);
        self.idx = (self.idx + 1) & self.layout.max_sequence();

        if self.idx == 0 {
            self.last_time_millis += 1;
        }

        id
    }

    #[inline]
    pub(crate) fn assemble(&self, timestamp_millis: i64) -> i64 {
        let layout = &self.layout;
        (timestamp_millis << layout.timestamp_shift)
            | ((self.machine_id as i64) << layout.machine_id_shift)
            | ((self.node_id as i64) << layout.node_id_shift)
            | (self.idx as i64)
    }
}
