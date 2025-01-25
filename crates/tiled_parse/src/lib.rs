//! Attempts to parse Tiled maps as they appear logically, rather than a one-to-one correspondence
//! with their `tmx` representation.

pub mod data_types;
pub mod parse;
pub mod relations;
pub(crate) mod util;
