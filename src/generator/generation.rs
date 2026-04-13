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
    pub fn generate(&mut self) -> Result<i64, SnowflakeError> {
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

        Ok(self.assemble(self.last_time_millis))
    }

    /// Generates the next ID, **always** reading the current clock.
    pub fn real_time_generate(&mut self) -> Result<i64, SnowflakeError> {
        self.idx = (self.idx + 1) & self.layout.max_sequence();

        let mut now = get_time_millis(self.epoch)?;

        if now < self.last_time_millis {
            return Err(SnowflakeError::ClockMovedBackwards);
        }

        if now == self.last_time_millis {
            if self.idx == 0 {
                now = biding_time_conditions(self.last_time_millis, self.epoch)?;
                self.last_time_millis = now;
            }
        } else {
            self.last_time_millis = now;
            self.idx = 0;
        }

        Ok(self.assemble(self.last_time_millis))
    }

    /// Generates the next ID without reading the system clock.
    pub fn lazy_generate(&mut self) -> i64 {
        self.idx = (self.idx + 1) & self.layout.max_sequence();

        if self.idx == 0 {
            self.last_time_millis += 1;
        }

        self.assemble(self.last_time_millis)
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
