use snowflake_gen::{BitLayout, init, next_id};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Instant;

fn main() {
    // 1. Initialize for a machine with a 5/5 sub-node layout (32 threads)
    let layout = BitLayout::new(41, 5, 5, 12).unwrap();
    init(1, layout).expect("could not initialize global snowflake");

    println!("Starting benchmark with 32 threads...");
    let thread_count = 32;
    let iterations_per_thread = 1_000_000;

    let counter = Arc::new(AtomicU64::new(0));
    let start = Instant::now();

    let handles: Vec<_> = (0..thread_count)
        .map(|_| {
            let cnt = Arc::clone(&counter);
            thread::spawn(move || {
                let mut local_count = 0;
                for _ in 0..iterations_per_thread {
                    if let Ok(_) = next_id() {
                        local_count += 1;
                    }
                }
                cnt.fetch_add(local_count, Ordering::Relaxed);
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let duration = start.elapsed();
    let total_ids = counter.load(Ordering::Relaxed);
    let ids_per_sec = total_ids as f64 / duration.as_secs_f64();

    println!("Generated {} IDs in {:?}", total_ids, duration);
    println!(
        "Throughput: {:.2} million IDs / sec",
        ids_per_sec / 1_000_000.0
    );
}
