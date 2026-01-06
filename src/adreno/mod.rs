//! Qualcomm Adreno GPU query module
//! 
//! This module provides functionality to query Qualcomm Adreno GPU information
//! via KGSL kernel driver ioctls on Linux/Android systems.

// Re-export public API
pub use query::{query_adreno, query_adreno_with_mode, query_adreno_robust};

// Internal modules
mod ioctl;
mod ioctl_impl;
mod database;
mod query;

/// Operation mode for Adreno GPUs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Parity mode - matches existing libgpuinfo behavior (lenient)
    Parity,
    /// Extended mode - full validation and additional features
    Extended,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Parity
    }
}

/// Database access functions
pub use database::{find_adreno_specs, AdrenoSpecs, SpecConfidence, AdrenoArch};

/// Ioctl structures
pub use ioctl::{
    KgslDeviceGetProperty, KgslDeviceInfo, KgslPropertyType,
};

/// Ioctl implementation functions
pub use ioctl_impl::{
    get_device_info, get_property, detect_working_ioctl,
};

#[cfg(feature = "debug")]
pub use query::debug_device_info;