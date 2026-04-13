use crate::generator::SnowflakeIdGenerator;
use crate::generator::components::SnowflakeComponents;

impl SnowflakeIdGenerator {
    /// Decodes a previously generated ID back into its constituent fields.
    ///
    /// ```rust
    /// use snowflake_gen::SnowflakeIdGenerator;
    ///
    /// let mut idgen = SnowflakeIdGenerator::new(3, 7).unwrap();
    /// let id = idgen.generate().unwrap();
    /// let parts = idgen.decompose(id);
    ///
    /// assert_eq!(parts.machine_id, 3);
    /// assert_eq!(parts.node_id, 7);
    /// ```
    pub fn decompose(&self, id: i64) -> SnowflakeComponents {
        let layout = &self.layout;

        let sequence_mask = layout.max_sequence() as i64;
        let node_mask = layout.max_node_id() as i64;
        let machine_mask = layout.max_machine_id() as i64;

        let sequence = (id & sequence_mask) as u32;
        let node_id = ((id >> layout.node_id_shift) & node_mask) as i32;
        let machine_id = ((id >> layout.machine_id_shift) & machine_mask) as i32;
        let timestamp_millis = id >> layout.timestamp_shift;

        SnowflakeComponents {
            timestamp_millis,
            machine_id,
            node_id,
            sequence,
        }
    }
}
