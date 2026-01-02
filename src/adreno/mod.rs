//! Qualcomm Adreno GPU query module
//!
//! This module provides functionality to query Qualcomm Adreno GPU information
//! via KGSL kernel ioctls on Linux/Android systems.

mod query;
mod database;

pub use query::query_adreno;
pub use database::{AdrenoSpecs, find_adreno_specs, SpecConfidence};