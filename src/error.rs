use thiserror::Error;

/// All errors that the snowflake crate can produce.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SnowflakeError {
    /// The bit widths of all four fields do not sum to 63.
    ///
    /// An `i64` has one sign bit, so the remaining 63 bits must be fully
    /// allocated to avoid negative IDs or wasted space.
    #[error(
        "bit fields must sum to exactly 63, \
         got timestamp({t}) + machine_id({m}) + node_id({n}) + sequence({s}) = {total}"
    )]
    InvalidBitLayout {
        t: u8,
        m: u8,
        n: u8,
        s: u8,
        total: u8,
    },

    /// A `machine_id` was provided that exceeds the layout's maximum.
    #[error("machine_id {given} exceeds maximum {max} for the configured bit width")]
    MachineIdOutOfRange { given: i64, max: i64 },

    /// A `node_id` was provided that exceeds the layout's maximum.
    #[error("node_id {given} exceeds maximum {max} for the configured bit width")]
    NodeIdOutOfRange { given: i64, max: i64 },

    /// The global generator has already been initialized.
    ///
    /// [`init`](crate::init) / [`init_with_epoch`](crate::init_with_epoch) can
    /// only be called once per process.  Re-initialization is rejected because
    /// existing thread-local generators would keep using the old configuration,
    /// leading to inconsistent ID layouts.  To change the configuration, restart
    /// the process so that all thread-local state is cleared.
    #[error("global snowflake generator is already initialized; restart the process to reconfigure")]
    AlreadyInitialized,

    /// More threads have called [`next_id`](crate::next_id) than the layout's
    /// `node_id_bits` can accommodate.
    ///
    /// With the default 5 `node_id_bits`, at most 32 threads (node IDs 0..=31)
    /// can generate IDs.  If your application needs more threads, increase
    /// `node_id_bits` (and reduce `machine_id_bits` or `timestamp_bits` to
    /// compensate) when calling [`init`](crate::init).
    #[error(
        "thread count exceeded the maximum node_id ({max}); \
         increase node_id_bits in your BitLayout to support more threads"
    )]
    NodeIdExhausted { max: i64 },

    /// The system clock moved backwards, which would break ID monotonicity.
    #[error("system clock moved backwards; cannot guarantee monotonic IDs")]
    ClockMovedBackwards,

    /// The system clock is not available (e.g. time before epoch).
    #[error("system clock error: time is before the configured epoch")]
    ClockBeforeEpoch,
}
