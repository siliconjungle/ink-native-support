//! Self-contained generated runtime support; no parser, evaluator or backend dependency.
pub const TYPED_VALUES_RUST_SOURCE: &str = include_str!("typed-values.rs.txt");
pub const OWNED_VALUES_RUST_SOURCE: &str = concat!(include_str!("owned-values.rs.txt"),include_str!("paged-columns.rs.txt"));
pub const DYNAMIC_VALUES_RUST_SOURCE: &str = include_str!("dynamic-values.rs.txt");
pub const STATE_EXPORTS_RUST_SOURCE: &str = include_str!("state-exports.rs.txt");
pub const REPRESENTATION_EXPORTS_RUST_SOURCE: &str = include_str!("representation-exports.rs.txt");

pub const INDEXED_TABLE_RUST_SOURCE: &str = include_str!("indexed-table.rs.txt");
pub const CONTINUATION_FRAMES_RUST_SOURCE: &str = include_str!("continuation-frames.rs.txt");

pub const BINARY_VALUES_RUST_SOURCE: &str = include_str!("binary-values.rs.txt");

pub const BINARY_EXPORTS_RUST_SOURCE: &str = include_str!("binary-exports.rs.txt");
pub const DIRECT_VALUES_RUST_SOURCE: &str = include_str!("direct-values.rs.txt");

pub const DYNAMIC_BINARY_RUST_SOURCE: &str = include_str!("dynamic-binary.rs.txt");

/// Generated fixed integer/float conversions; needs the host BigInt dependency.
pub const NUMERIC_INT_RUST_SOURCE:&str=include_str!("numeric-int.rs.txt");
pub const TYPED_SNAPSHOT_RUST_SOURCE: &str = include_str!("typed-snapshot.rs.txt");
