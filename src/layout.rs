use crate::error::SnowflakeError;

/// Describes how the 63 usable bits of an `i64` snowflake ID are partitioned.
///
/// All four fields must sum to exactly **63**.  The sign bit is reserved so
/// that all generated IDs are positive and can be stored in signed integer
/// columns without surprises.
///
/// # Field ordering (MSB → LSB)
///
/// ```text
/// [sign(1)] [timestamp(T)] [machine_id(M)] [node_id(N)] [sequence(S)]
/// ```
///
/// # Example
///
/// ```rust
/// use snowflake_gen::BitLayout;
///
/// // Classic Twitter layout
/// let layout = BitLayout::default();
/// assert_eq!(layout.max_sequence(), 4095);
/// assert_eq!(layout.max_machine_id(), 31);
/// assert_eq!(layout.max_node_id(), 31);
/// ```
#[must_use]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BitLayout {
    /// Number of bits allocated to the millisecond timestamp.
    pub(crate) timestamp_bits: u8,
    /// Number of bits allocated to the machine identifier.
    pub(crate) machine_id_bits: u8,
    /// Number of bits allocated to the node (worker) identifier.
    pub(crate) node_id_bits: u8,
    /// Number of bits allocated to the per-millisecond sequence counter.
    pub(crate) sequence_bits: u8,

    // Precomputed shift offsets (derived from the widths above).
    pub(crate) node_id_shift: u8,
    pub(crate) machine_id_shift: u8,
    pub(crate) timestamp_shift: u8,
}

impl BitLayout {
    /// Creates a validated `BitLayout`.
    ///
    /// Returns [`SnowflakeError::InvalidBitLayout`] if the four bit widths do
    /// not sum to exactly 63.
    ///
    /// # Example
    ///
    /// ```rust
    /// use snowflake_gen::BitLayout;
    ///
    /// let layout = BitLayout::new(41, 5, 5, 12).unwrap(); // classic Twitter
    /// let bad    = BitLayout::new(40, 5, 5, 12);          // sums to 62 → error
    /// assert!(bad.is_err());
    /// ```
    pub fn new(
        timestamp_bits: u8,
        machine_id_bits: u8,
        node_id_bits: u8,
        sequence_bits: u8,
    ) -> Result<Self, SnowflakeError> {
        let total = timestamp_bits
            .checked_add(machine_id_bits)
            .and_then(|s| s.checked_add(node_id_bits))
            .and_then(|s| s.checked_add(sequence_bits))
            .unwrap_or(u8::MAX);

        if total != 63 {
            return Err(SnowflakeError::InvalidBitLayout {
                t: timestamp_bits,
                m: machine_id_bits,
                n: node_id_bits,
                s: sequence_bits,
                total,
            });
        }

        let node_id_shift = sequence_bits;
        let machine_id_shift = node_id_shift + node_id_bits;
        let timestamp_shift = machine_id_shift + machine_id_bits;

        Ok(Self {
            timestamp_bits,
            machine_id_bits,
            node_id_bits,
            sequence_bits,
            node_id_shift,
            machine_id_shift,
            timestamp_shift,
        })
    }

    /// Returns the number of bits allocated to the timestamp.
    #[inline]
    pub const fn timestamp_bits(&self) -> u8 {
        self.timestamp_bits
    }

    /// Returns the number of bits allocated to the machine identifier.
    #[inline]
    pub const fn machine_id_bits(&self) -> u8 {
        self.machine_id_bits
    }

    /// Returns the number of bits allocated to the node identifier.
    #[inline]
    pub const fn node_id_bits(&self) -> u8 {
        self.node_id_bits
    }

    /// Returns the number of bits allocated to the sequence counter.
    #[inline]
    pub const fn sequence_bits(&self) -> u8 {
        self.sequence_bits
    }

    /// Maximum allowed sequence value (IDs per millisecond − 1).
    ///
    /// ```rust
    /// use snowflake_gen::BitLayout;
    /// assert_eq!(BitLayout::default().max_sequence(), 4095);
    /// ```
    #[inline]
    pub const fn max_sequence(&self) -> u32 {
        (1u32 << self.sequence_bits) - 1
    }

    /// Maximum allowed `machine_id`.
    ///
    /// ```rust
    /// use snowflake_gen::BitLayout;
    /// assert_eq!(BitLayout::default().max_machine_id(), 31);
    /// ```
    #[inline]
    pub const fn max_machine_id(&self) -> i64 {
        (1i64 << self.machine_id_bits) - 1
    }

    /// Maximum allowed `node_id`.
    ///
    /// ```rust
    /// use snowflake_gen::BitLayout;
    /// assert_eq!(BitLayout::default().max_node_id(), 31);
    /// ```
    #[inline]
    pub const fn max_node_id(&self) -> i64 {
        (1i64 << self.node_id_bits) - 1
    }

    /// Maximum milliseconds the timestamp field can represent.
    ///
    /// With 41 timestamp bits this is ~69 years from epoch.
    #[inline]
    pub const fn max_timestamp_millis(&self) -> i64 {
        (1i64 << self.timestamp_bits) - 1
    }

    /// Maximum IDs that can be generated per second across all nodes.
    #[inline]
    pub fn max_ids_per_second(&self) -> u64 {
        (self.max_sequence() as u64 + 1)
            * (self.max_machine_id() as u64 + 1)
            * (self.max_node_id() as u64 + 1)
            * 1_000
    }
}

impl Default for BitLayout {
    fn default() -> Self {
        Self::new(41, 5, 5, 12).expect("default layout is always valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_layout_is_valid() {
        let l = BitLayout::default();
        assert_eq!(l.max_sequence(), 4095);
        assert_eq!(l.max_machine_id(), 31);
        assert_eq!(l.max_node_id(), 31);
    }

    #[test]
    fn custom_layout_capacities() {
        let l = BitLayout::new(38, 8, 7, 10).unwrap();
        assert_eq!(l.max_sequence(), 1023);
        assert_eq!(l.max_machine_id(), 255);
        assert_eq!(l.max_node_id(), 127);
    }

    #[test]
    fn invalid_bit_layout_rejected() {
        assert!(BitLayout::new(40, 5, 5, 12).is_err());
    }

    #[test]
    fn max_timestamp_millis() {
        let l = BitLayout::default();
        // 2^41 - 1 = 2_199_023_255_551 (~69 years in ms)
        assert_eq!(l.max_timestamp_millis(), 2_199_023_255_551);
    }

    #[test]
    fn max_ids_per_second() {
        let l = BitLayout::default();
        // (4095 + 1) * (31 + 1) * (31 + 1) * 1000
        assert_eq!(l.max_ids_per_second(), 4_096 * 32 * 32 * 1_000);
    }

    #[test]
    fn zero_width_fields() {
        // All bits to timestamp and sequence, zero machine/node
        let l = BitLayout::new(51, 0, 0, 12).unwrap();
        assert_eq!(l.max_machine_id(), 0);
        assert_eq!(l.max_node_id(), 0);
        assert_eq!(l.max_sequence(), 4095);
    }

    #[test]
    fn max_sequence_bits_after_widening() {
        // 31 sequence bits — the widened u32 return type must handle this
        let l = BitLayout::new(16, 8, 8, 31).unwrap();
        assert_eq!(l.max_sequence(), (1u32 << 31) - 1);
    }
}
