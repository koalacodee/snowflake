use super::*;
use crate::error::SnowflakeError;
use crate::layout::BitLayout;

#[test]
fn machine_id_out_of_range() {
    let err = SnowflakeIdGenerator::new(32, 0).unwrap_err();
    assert!(matches!(err, SnowflakeError::MachineIdOutOfRange { .. }));
}

#[test]
fn monotonicity() {
    let mut idgen = SnowflakeIdGenerator::new(1, 1).unwrap();
    let mut prev = idgen.generate().unwrap();
    for _ in 0..10_000 {
        let next = idgen.generate().unwrap();
        assert!(next > prev);
        prev = next;
    }
}

#[test]
fn round_trip() {
    let mut idgen = SnowflakeIdGenerator::new(5, 11).unwrap();
    let id = idgen.generate().unwrap();
    let parts = idgen.decompose(id);
    assert_eq!(parts.machine_id, 5);
    assert_eq!(parts.node_id, 11);
}

#[test]
fn decompose_custom_layout() {
    let layout = BitLayout::new(38, 8, 7, 10).unwrap();
    let mut idgen = SnowflakeIdGenerator::with_layout(100, 60, layout).unwrap();
    let id = idgen.generate().unwrap();
    let parts = idgen.decompose(id);

    assert_eq!(parts.machine_id, 100);
    assert_eq!(parts.node_id, 60);
}
