//! Unified GPU Information Query Library
//!
//! This library provides a unified interface to query GPU information
//! for both ARM Mali and Qualcomm Adreno GPUs on Linux/Android systems.
pub use info::GpuInfoBuilder;  
// Common modules
pub mod error;
pub mod info;

// Conditionally compiled modules
#[cfg(feature = "mali")]
pub mod mali;

#[cfg(feature = "adreno")]
pub mod adreno;


#[cfg(feature = "auto-detect")]
pub mod detect;

// Re-export common types
pub use error::{GpuError, GpuResult};
pub use info::{GpuInfo, GpuVendor, MaliData, AdrenoData};

/// Operation mode for Mali GPUs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Best-effort libgpuinfo semantics.
    Parity,
    /// Full feature implementation with validation.
    Extended,
}

// Mali-specific API (conditionally compiled)
#[cfg(feature = "mali")]
pub use mali::{query_mali, query_mali_with_mode};

// Adreno-specific API (conditionally compiled)
#[cfg(feature = "adreno")]
pub use adreno::query_adreno;

// Auto-detection API (conditionally compiled)
#[cfg(feature = "auto-detect")]
pub use detect::query_gpu_auto;

// Legacy API for backward compatibility (Mali-specific)
#[cfg(feature = "mali")]
pub fn query_gpu<P: AsRef<std::path::Path>>(device_path: P) -> GpuResult<GpuInfo> {
    query_mali(device_path)
}

#[cfg(feature = "mali")]
pub fn query_gpu_with_mode<P: AsRef<std::path::Path>>(
    device_path: P,
    mode: Mode
) -> GpuResult<GpuInfo> {
    query_mali_with_mode(device_path, mode)
}

/// Unified query function (requires auto-detect feature)
#[cfg(feature = "auto-detect")]
pub fn query_gpu_unified<P: AsRef<std::path::Path>>(
    device_path: Option<P>
) -> GpuResult<GpuInfo> {
    query_gpu_auto(device_path)
}

pub trait IntoCow {
    fn into_cow(self) -> std::borrow::Cow<'static, str>;
}

impl IntoCow for &'static str {
    fn into_cow(self) -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(self)
    }
}

impl IntoCow for String {
    fn into_cow(self) -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Owned(self)
    }
}