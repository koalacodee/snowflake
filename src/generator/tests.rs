use super::*;
use crate::error::SnowflakeError;
use crate::layout::BitLayout;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ── Construction validation ──────────────────────────────────────────

#[test]
fn machine_id_out_of_range() {
    let err = SnowflakeIdGenerator::new(32, 0).unwrap_err();
    assert!(matches!(err, SnowflakeError::MachineIdOutOfRange { .. }));
}

#[test]
fn negative_machine_id_rejected() {
    let err = SnowflakeIdGenerator::new(-1, 0).unwrap_err();
    assert!(matches!(err, SnowflakeError::MachineIdOutOfRange { given: -1, .. }));
}

#[test]
fn node_id_out_of_range() {
    let err = SnowflakeIdGenerator::new(0, 32).unwrap_err();
    assert!(matches!(err, SnowflakeError::NodeIdOutOfRange { .. }));
}

#[test]
fn negative_node_id_rejected() {
    let err = SnowflakeIdGenerator::new(0, -1).unwrap_err();
    assert!(matches!(err, SnowflakeError::NodeIdOutOfRange { given: -1, .. }));
}

#[test]
fn epoch_in_future_rejected() {
    let future = SystemTime::now() + Duration::from_secs(3600);
    let err = SnowflakeIdGenerator::with_epoch(0, 0, future).unwrap_err();
    assert!(matches!(err, SnowflakeError::ClockBeforeEpoch));
}

#[test]
fn custom_epoch_accepted() {
    // Discord epoch: 1 January 2015
    let epoch = UNIX_EPOCH + Duration::from_millis(1_420_070_400_000);
    let mut idgen= SnowflakeIdGenerator::with_epoch(1, 1, epoch).unwrap();
    let id = idgen.generate().unwrap();
    assert!(id > 0);
}

// ── generate() ───────────────────────────────────────────────────────

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
fn first_id_uses_sequence_zero() {
    let mut idgen= SnowflakeIdGenerator::new(0, 0).unwrap();
    let id = idgen.generate().unwrap();
    let parts = idgen.decompose(id);
    assert_eq!(parts.sequence, 0);
}

#[test]
fn sequence_exhaustion_advances_timestamp() {
    let mut idgen= SnowflakeIdGenerator::new(0, 0).unwrap();
    let first = idgen.generate().unwrap();
    let first_parts = idgen.decompose(first);

    // Exhaust the full sequence (0..=max_sequence)
    let max_seq = idgen.layout().max_sequence() as usize;
    for _ in 1..=max_seq {
        let _ = idgen.generate().unwrap();
    }

    // Next ID must have a later timestamp
    let after = idgen.generate().unwrap();
    let after_parts = idgen.decompose(after);
    assert!(
        after_parts.timestamp_millis > first_parts.timestamp_millis,
        "timestamp should advance after sequence exhaustion"
    );
    assert_eq!(after_parts.sequence, 0);
}

// ── real_time_generate() ─────────────────────────────────────────────

#[test]
fn real_time_generate_monotonicity() {
    let mut idgen= SnowflakeIdGenerator::new(1, 1).unwrap();
    let mut prev = idgen.real_time_generate().unwrap();
    for _ in 0..10_000 {
        let next = idgen.real_time_generate().unwrap();
        assert!(next > prev);
        prev = next;
    }
}

#[test]
fn real_time_generate_uniqueness() {
    let mut idgen= SnowflakeIdGenerator::new(1, 1).unwrap();
    let mut ids: Vec<i64> = (0..5_000).map(|_| idgen.real_time_generate().unwrap()).collect();
    let len = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), len, "all IDs must be unique");
}

// ── lazy_generate() ──────────────────────────────────────────────────

#[test]
fn lazy_generate_uniqueness() {
    let mut idgen= SnowflakeIdGenerator::new(1, 1).unwrap();
    let mut ids: Vec<i64> = (0..10_000).map(|_| idgen.lazy_generate()).collect();
    let len = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), len, "all lazy IDs must be unique");
}

#[test]
fn lazy_generate_monotonicity() {
    let mut idgen= SnowflakeIdGenerator::new(1, 1).unwrap();
    let mut prev = idgen.lazy_generate();
    for _ in 0..10_000 {
        let next = idgen.lazy_generate();
        assert!(next > prev);
        prev = next;
    }
}

#[test]
fn lazy_generate_bumps_timestamp_on_wrap() {
    let mut idgen= SnowflakeIdGenerator::new(0, 0).unwrap();
    let first = idgen.lazy_generate();
    let first_ts = idgen.decompose(first).timestamp_millis;

    let max_seq = idgen.layout().max_sequence() as usize;
    for _ in 1..=max_seq {
        let _ = idgen.lazy_generate();
    }

    // Sequence just wrapped — next ID should have timestamp + 1
    let after = idgen.lazy_generate();
    let after_ts = idgen.decompose(after).timestamp_millis;
    assert_eq!(after_ts, first_ts + 1);
}

// ── Decomposition ────────────────────────────────────────────────────

#[test]
fn round_trip() {
    let mut idgen = SnowflakeIdGenerator::new(5, 11).unwrap();
    let id = idgen.generate().unwrap();
    let parts = idgen.decompose(id);
    assert_eq!(parts.machine_id, 5);
    assert_eq!(parts.node_id, 11);
}

#[test]
fn round_trip_includes_sequence() {
    let mut idgen= SnowflakeIdGenerator::new(1, 1).unwrap();

    // First ID → sequence 0
    let id0 = idgen.generate().unwrap();
    assert_eq!(idgen.decompose(id0).sequence, 0);

    // Second ID → sequence 1
    let id1 = idgen.generate().unwrap();
    assert_eq!(idgen.decompose(id1).sequence, 1);
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

#[test]
fn decompose_with_custom_epoch() {
    let epoch = UNIX_EPOCH + Duration::from_millis(1_420_070_400_000);
    let mut idgen= SnowflakeIdGenerator::with_epoch(3, 7, epoch).unwrap();
    let id = idgen.generate().unwrap();
    let parts = idgen.decompose(id);

    assert_eq!(parts.machine_id, 3);
    assert_eq!(parts.node_id, 7);
    assert!(parts.timestamp_millis > 0);
}
