# snowflake-gen

A configurable [Snowflake ID](https://en.wikipedia.org/wiki/Snowflake_ID) generator for Rust.

## Features

- **Configurable bit layout** -- tune the balance between timestamp range, throughput, and node count
- **Thread-local global API** -- zero-lock, zero-contention ID generation across threads
- **ID decomposition** -- decode any generated ID back into its timestamp, machine, node, and sequence parts
- **Buffered generation** -- `SnowflakeIdBucket` pre-generates a full sequence batch for maximum throughput
- **Custom epochs** -- use Discord-style, Twitter-style, or your own epoch

### Default layout (Twitter-compatible)

| Field | Bits | Max value |
|------------|------|-----------|
| Timestamp | 41 | ~69 years |
| Machine ID | 5 | 31 |
| Node ID | 5 | 31 |
| Sequence | 12 | 4,095/ms |

## Quick start

```rust
use snowflake_gen::SnowflakeIdGenerator;

let mut gen = SnowflakeIdGenerator::new(1, 1).unwrap();
let id = gen.generate().unwrap();
println!("{id}");
```

## Custom bit layout

```rust
use snowflake_gen::{BitLayout, SnowflakeIdGenerator};

// 10 sequence bits (1,023 IDs/ms), 8 machine + 7 node bits
let layout = BitLayout::new(38, 8, 7, 10).unwrap();
let mut gen = SnowflakeIdGenerator::with_layout(1, 1, layout).unwrap();
let id = gen.generate().unwrap();
```

## Thread-local global API

Initialize once, then call `next_id()` from any thread with no locking:

```rust
use snowflake_gen::{BitLayout, init, next_id};

fn main() {
    init(1, BitLayout::default()).unwrap();

    let id = next_id().unwrap();
    println!("{id}");
}
```

Each thread gets its own generator with a unique `node_id` assigned automatically.

## ID decomposition

```rust
use snowflake_gen::SnowflakeIdGenerator;

let mut gen = SnowflakeIdGenerator::new(3, 7).unwrap();
let id = gen.generate().unwrap();
let parts = gen.decompose(id);

assert_eq!(parts.machine_id, 3);
assert_eq!(parts.node_id, 7);
```

## Buffered generation

`SnowflakeIdBucket` pre-generates a full sequence cycle and serves IDs from a buffer:

```rust
use snowflake_gen::SnowflakeIdBucket;

let mut bucket = SnowflakeIdBucket::new(1, 1).unwrap();
let id = bucket.get_id();
```

## Caveats

- **Do not mix `lazy_generate` with clock-based methods** (`generate` / `real_time_generate`) on the same generator instance. `lazy_generate` advances the internal timestamp synthetically, so a later clock-based call may reuse a timestamp that `lazy_generate` already claimed, producing duplicate IDs. `SnowflakeIdBucket` uses `lazy_generate` internally on a dedicated generator and is safe to use alongside separate clock-based generators.

## License

MIT
