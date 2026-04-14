use std::time::SystemTime;
use crate::layout::BitLayout;

pub mod components;
pub mod creation;
pub mod generation;
pub mod decomposition;

#[cfg(test)]
pub mod tests;

pub use components::SnowflakeComponents;

/// A Snowflake ID generator with a configurable [`BitLayout`].
///
/// # Thread safety
///
/// `SnowflakeIdGenerator` is **not** `Send + Sync` by itself because its
/// internal state is mutated on every call. Wrap it in a `Mutex` / `RwLock`
/// when sharing across threads.
///
/// # Example — default layout
///
/// ```rust
/// use snowflake_gen::SnowflakeIdGenerator;
///
/// let mut idgen = SnowflakeIdGenerator::new(1, 1).unwrap();
/// let id = idgen.generate().unwrap();
/// assert!(id > 0);
/// ```
#[derive(Copy, Clone, Debug)]
pub struct SnowflakeIdGenerator {
    /// Epoch used for timestamp calculations.
    pub(crate) epoch: SystemTime,
    /// Timestamp of the last generated ID (millis since epoch).
    pub(crate) last_time_millis: i64,
    /// The machine identifier baked into every ID.
    pub(crate) machine_id: i32,
    /// The node (worker) identifier baked into every ID.
    pub(crate) node_id: i32,
    /// Per-millisecond auto-increment counter.
    pub(crate) idx: u32,
    /// Bit layout governing field widths and shifts.
    pub(crate) layout: BitLayout,
}

impl SnowflakeIdGenerator {
    /// Returns a reference to the [`BitLayout`] in use.
    #[inline]
    pub fn layout(&self) -> &BitLayout {
        &self.layout
    }

    /// Returns the epoch this generator was constructed with.
    #[inline]
    pub fn epoch(&self) -> SystemTime {
        self.epoch
    }

    /// Returns the machine identifier.
    #[inline]
    pub fn machine_id(&self) -> i32 {
        self.machine_id
    }

    /// Returns the node (worker) identifier.
    #[inline]
    pub fn node_id(&self) -> i32 {
        self.node_id
    }
}
