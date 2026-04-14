//! # snowflake-gen
//!
//! A configurable Snowflake ID generator for Rust.
//!
//! ## Overview
//!
//! This crate implements the [Snowflake](https://en.wikipedia.org/wiki/Snowflake_ID) algorithm
//! with fully configurable bit allocation so you can tune the balance between:
//!
//! - **Timestamp range** — how far into the future IDs remain valid
//! - **Throughput** — how many IDs per millisecond (sequence bits)
//! - **Node / machine count** — how many distributed generators you can run
//!
//! The classic Twitter layout is used by default:
//!
//! | Field       | Bits | Max value          |
//! |-------------|------|--------------------|
//! | Timestamp   | 41   | ~69 years           |
//! | Machine ID  | 5    | 31                 |
//! | Node ID     | 5    | 31                 |
//! | Sequence    | 12   | 4 095 / ms         |
//!
//! ## Quick start
//!
//! ```rust
//! use snowflake_gen::SnowflakeIdGenerator;
//!
//! let mut idgen = SnowflakeIdGenerator::new(1, 1).unwrap();
//! let id = idgen.generate().unwrap();
//! assert!(id > 0);
//! ```
//!
//! ## Custom layout
//!
//! ```rust
//! use snowflake_gen::{BitLayout, SnowflakeIdGenerator};
//!
//! // 10 sequence bits → 1 023 IDs/ms, 8 machine + 7 node bits
//! let layout = BitLayout::new(38, 8, 7, 10).unwrap();
//! let mut idgen = SnowflakeIdGenerator::with_layout(1, 1, layout).unwrap();
//! let id = idgen.generate().unwrap();
//! assert!(id > 0);
//! ```

pub mod bucket;
pub mod error;
pub mod generator;
pub mod global;
pub mod layout;
pub(crate) mod utils;

pub use bucket::SnowflakeIdBucket;
pub use error::SnowflakeError;
pub use generator::{SnowflakeComponents, SnowflakeIdGenerator};
pub use global::{init, init_with_epoch, is_initialized, next_id, real_time_next_id};
pub use layout::BitLayout;

/// Convenience re-exports.
pub mod prelude {
    pub use super::{
        BitLayout, SnowflakeComponents, SnowflakeError, SnowflakeIdBucket, SnowflakeIdGenerator,
    };
}
