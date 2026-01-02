use std::path::Path;

use crate::error::{GpuError, GpuResult};
use crate::info::GpuInfo;

/// Automatically detect and query GPU
#[cfg(all(feature = "auto-detect", any(feature = "mali", feature = "adreno")))]
pub fn query_gpu_auto<P: AsRef<std::path::Path>>(device_path: Option<P>) -> GpuResult<GpuInfo> {
    // Try Mali first if device path is provided or default exists
    #[cfg(feature = "mali")]
    {
        // REMOVED: use crate::Mode; // Not needed here

        if let Some(path) = &device_path {
            // FIXED: Use query_mali_with_mode with explicit Mode::Parity
            if let Ok(info) = crate::mali::query_mali_with_mode(path, crate::Mode::Parity) {
                return Ok(info);
            }
        } else if Path::new("/dev/mali0").exists() {
            // FIXED: Use query_mali_with_mode with explicit Mode::Parity
            if let Ok(info) = crate::mali::query_mali_with_mode("/dev/mali0", crate::Mode::Parity) {
                return Ok(info);
            }
        }
    }

    // Try Adreno if KGSL device exists
    #[cfg(feature = "adreno")]
    {
        if Path::new("/dev/kgsl-3d0").exists() {
            if let Ok(info) = crate::adreno::query_adreno("/dev/kgsl-3d0") {
                return Ok(info);
            }
        }
    }

    // No GPU found
    Err(GpuError::DeviceNotFound)
}