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
    MachineIdOutOfRange { given: i32, max: i32 },

    /// A `node_id` was provided that exceeds the layout's maximum.
    #[error("node_id {given} exceeds maximum {max} for the configured bit width")]
    NodeIdOutOfRange { given: i32, max: i32 },

    /// The system clock moved backwards, which would break ID monotonicity.
    #[error("system clock moved backwards; cannot guarantee monotonic IDs")]
    ClockMovedBackwards,

    /// The system clock is not available (e.g. time before epoch).
    #[error("system clock error: time is before the configured epoch")]
    ClockBeforeEpoch,
}
