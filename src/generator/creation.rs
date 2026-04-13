use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::SnowflakeError;
use crate::layout::BitLayout;
use crate::utils::get_time_millis;
use crate::generator::SnowflakeIdGenerator;

impl SnowflakeIdGenerator {
    /// Constructs a generator using the UNIX epoch and the default [`BitLayout`].
    ///
    /// ```rust
    /// use snowflake_gen::SnowflakeIdGenerator;
    ///
    /// let mut idgen = SnowflakeIdGenerator::new(1, 1).unwrap();
    /// ```
    pub fn new(machine_id: i32, node_id: i32) -> Result<Self, SnowflakeError> {
        Self::with_epoch(machine_id, node_id, UNIX_EPOCH)
    }

    /// Constructs a generator using a custom epoch and the default [`BitLayout`].
    ///
    /// ```rust
    /// use std::time::{Duration, UNIX_EPOCH};
    /// use snowflake_gen::SnowflakeIdGenerator;
    ///
    /// // Discord epoch: 1 January 2015 00:00:00 UTC
    /// let discord_epoch = UNIX_EPOCH + Duration::from_millis(1_420_070_400_000);
    /// let mut idgen = SnowflakeIdGenerator::with_epoch(1, 1, discord_epoch).unwrap();
    /// ```
    pub fn with_epoch(
        machine_id: i32,
        node_id: i32,
        epoch: SystemTime,
    ) -> Result<Self, SnowflakeError> {
        Self::with_layout_and_epoch(machine_id, node_id, BitLayout::default(), epoch)
    }

    /// Constructs a generator with a fully custom [`BitLayout`] and UNIX epoch.
    ///
    /// ```rust
    /// use snowflake_gen::{BitLayout, SnowflakeIdGenerator};
    ///
    /// let layout = BitLayout::new(38, 8, 7, 10).unwrap();
    /// let mut idgen = SnowflakeIdGenerator::with_layout(1, 1, layout).unwrap();
    /// ```
    pub fn with_layout(
        machine_id: i32,
        node_id: i32,
        layout: BitLayout,
    ) -> Result<Self, SnowflakeError> {
        Self::with_layout_and_epoch(machine_id, node_id, layout, UNIX_EPOCH)
    }

    /// Constructs a generator with a fully custom [`BitLayout`] and epoch.
    ///
    /// # Errors
    ///
    /// - [`SnowflakeError::MachineIdOutOfRange`] if `machine_id > layout.max_machine_id()`.
    /// - [`SnowflakeError::NodeIdOutOfRange`] if `node_id > layout.max_node_id()`.
    /// - [`SnowflakeError::ClockBeforeEpoch`] if the current time is before `epoch`.
    pub fn with_layout_and_epoch(
        machine_id: i32,
        node_id: i32,
        layout: BitLayout,
        epoch: SystemTime,
    ) -> Result<Self, SnowflakeError> {
        if machine_id < 0 || machine_id > layout.max_machine_id() {
            return Err(SnowflakeError::MachineIdOutOfRange {
                given: machine_id,
                max: layout.max_machine_id(),
            });
        }
        if node_id < 0 || node_id > layout.max_node_id() {
            return Err(SnowflakeError::NodeIdOutOfRange {
                given: node_id,
                max: layout.max_node_id(),
            });
        }

        let last_time_millis = get_time_millis(epoch)?;

        Ok(Self {
            epoch,
            last_time_millis,
            machine_id,
            node_id,
            idx: 0,
            layout,
        })
    }
}
