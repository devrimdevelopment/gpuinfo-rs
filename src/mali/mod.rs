//! ARM Mali GPU query module
//!
//! This module provides functionality to query ARM Mali GPU information
//! via kernel ioctls on Linux/Android systems.

mod query;
mod database;
mod parser;

pub use query::{query_mali, query_mali_with_mode};
pub use parser::{parse_properties, parse_properties_lenient, ParserConfig, ParsedProperties};

// Re-export the Mode enum for compatibility
pub use crate::Mode;