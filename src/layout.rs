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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BitLayout {
    /// Number of bits allocated to the millisecond timestamp.
    pub timestamp_bits: u8,
    /// Number of bits allocated to the machine identifier.
    pub machine_id_bits: u8,
    /// Number of bits allocated to the node (worker) identifier.
    pub node_id_bits: u8,
    /// Number of bits allocated to the per-millisecond sequence counter.
    pub sequence_bits: u8,

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

    /// Maximum allowed sequence value (IDs per millisecond − 1).
    ///
    /// ```rust
    /// use snowflake_gen::BitLayout;
    /// assert_eq!(BitLayout::default().max_sequence(), 4095);
    /// ```
    #[inline]
    pub const fn max_sequence(&self) -> u16 {
        (1u16 << self.sequence_bits) - 1
    }

    /// Maximum allowed `machine_id`.
    ///
    /// ```rust
    /// use snowflake_gen::BitLayout;
    /// assert_eq!(BitLayout::default().max_machine_id(), 31);
    /// ```
    #[inline]
    pub const fn max_machine_id(&self) -> i32 {
        (1i32 << self.machine_id_bits) - 1
    }

    /// Maximum allowed `node_id`.
    ///
    /// ```rust
    /// use snowflake_gen::BitLayout;
    /// assert_eq!(BitLayout::default().max_node_id(), 31);
    /// ```
    #[inline]
    pub const fn max_node_id(&self) -> i32 {
        (1i32 << self.node_id_bits) - 1
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
}
