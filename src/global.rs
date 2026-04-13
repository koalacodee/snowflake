use std::cell::RefCell;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use crate::error::SnowflakeError;
use crate::generator::SnowflakeIdGenerator;
use crate::layout::BitLayout;

use std::time::{SystemTime, UNIX_EPOCH};

static INITIALIZED: AtomicBool = AtomicBool::new(false);
static MACHINE_ID: AtomicI32 = AtomicI32::new(0);
static NODE_COUNTER: AtomicI32 = AtomicI32::new(0);
static LAYOUT: OnceLock<BitLayout> = OnceLock::new();
static EPOCH: OnceLock<SystemTime> = OnceLock::new();

thread_local! {
    /// thread-local generator for that thread. The number of allowed threads is limited
    /// by the `node_id_bits` in the [`BitLayout`].
    static LOCAL_GEN: RefCell<Option<SnowflakeIdGenerator>> = const { RefCell::new(None) };
}

/// Initializes the global ID generation system with the UNIX epoch.
///
/// Must be called **exactly once** (usually in `main`) before any calls to
/// [`next_id`].  Calling it a second time returns
/// [`SnowflakeError::AlreadyInitialized`].  To change the configuration,
/// restart the process so that all thread-local generators are dropped and
/// recreated with the new settings.
///
/// # Errors
///
/// - [`SnowflakeError::AlreadyInitialized`] if called more than once.
pub fn init(machine_id: i32, layout: BitLayout) -> Result<(), SnowflakeError> {
    init_with_epoch(machine_id, layout, UNIX_EPOCH)
}

/// Initializes the global ID generation system with a custom epoch.
///
/// Must be called **exactly once** (usually in `main`) before any calls to
/// [`next_id`].  Calling it a second time returns
/// [`SnowflakeError::AlreadyInitialized`].  To change the configuration,
/// restart the process so that all thread-local generators are dropped and
/// recreated with the new settings.
///
/// # Errors
///
/// - [`SnowflakeError::AlreadyInitialized`] if called more than once.
pub fn init_with_epoch(
    machine_id: i32,
    layout: BitLayout,
    epoch: SystemTime,
) -> Result<(), SnowflakeError> {
    if INITIALIZED.swap(true, Ordering::SeqCst) {
        return Err(SnowflakeError::AlreadyInitialized);
    }

    MACHINE_ID.store(machine_id, Ordering::SeqCst);
    LAYOUT.set(layout).ok();
    EPOCH.set(epoch).ok();

    Ok(())
}

/// Returns `true` if the global generator has been initialized.
pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::Relaxed)
}

/// Generates a new unique ID using a thread-local generator.
///
/// This is the highest performance way to generate IDs as it involves **zero locking**
/// and **zero contention** between threads.
///
/// # Panics
///
/// Panics if [`init`] has not been called.
pub fn next_id() -> Result<i64, SnowflakeError> {
    ensure_init();

    LOCAL_GEN.with(|cell| {
        let mut opt = cell.borrow_mut();
        let generator = opt.get_or_insert_with(create_generator);
        generator.generate()
    })
}

/// Generates a new unique ID using a thread-local generator, always reading the system clock.
///
/// # Panics
///
/// Panics if [`init`] has not been called.
pub fn real_time_next_id() -> Result<i64, SnowflakeError> {
    ensure_init();

    LOCAL_GEN.with(|cell| {
        let mut opt = cell.borrow_mut();
        let generator = opt.get_or_insert_with(create_generator);
        generator.real_time_generate()
    })
}

fn create_generator() -> SnowflakeIdGenerator {
    let m_id = MACHINE_ID.load(Ordering::Relaxed);
    let n_id = NODE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let layout = *LAYOUT.get().unwrap_or(&BitLayout::default());
    let epoch = *EPOCH.get().unwrap_or(&UNIX_EPOCH);

    SnowflakeIdGenerator::with_layout_and_epoch(m_id, n_id, layout, epoch)
        .expect("global settings must be valid")
}

#[inline]
fn ensure_init() {
    if !is_initialized() {
        panic!("Snowflake system must be initialized with `snowflake::init(...)` before use.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_global_initialization() {
        // Note: Tests run in parallel, and since this is global state,
        // we can't easily reset it. We just test that it works once.
        let layout = BitLayout::default();
        let _ = init(1, layout);
        assert!(is_initialized());
    }

    #[test]
    fn test_next_id_concurrently() {
        let _ = init(1, BitLayout::default());

        let handles: Vec<_> = (0..10)
            .map(|_| {
                thread::spawn(|| {
                    let id = next_id().expect("should generate ID");
                    assert!(id > 0);
                    id
                })
            })
            .collect();

        let mut ids = Vec::new();
        for h in handles {
            ids.push(h.join().unwrap());
        }

        let mut sorted_ids = ids.clone();
        sorted_ids.sort();
        sorted_ids.dedup();

        assert_eq!(
            ids.len(),
            sorted_ids.len(),
            "IDs should be unique across threads"
        );
    }

    #[test]
    fn test_real_time_next_id() {
        let _ = init(1, BitLayout::default());
        let id = real_time_next_id().expect("should generate ID");
        assert!(id > 0);
    }
}
