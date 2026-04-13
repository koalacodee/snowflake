/// The decoded components of a Snowflake ID.
///
/// Obtained via [`crate::generator::SnowflakeIdGenerator::decompose`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SnowflakeComponents {
    /// Milliseconds since the generator's epoch.
    pub timestamp_millis: i64,
    /// Machine identifier embedded in the ID.
    pub machine_id: i32,
    /// Node identifier embedded in the ID.
    pub node_id: i32,
    /// Per-millisecond sequence counter embedded in the ID.
    pub sequence: u16,
}
