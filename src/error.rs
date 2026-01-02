/// Error type for GPU information queries
///
/// This enum is marked as #[non_exhaustive] to allow adding new error variants
/// in the future without breaking existing code.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum GpuError {
    /// I/O error (file not found, permission denied, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Ioctl operation failed
    #[error("Ioctl operation {request:#x} failed: {source}")]
    IoctlFailed {
        /// The ioctl request number that failed
        request: u64,
        /// The underlying I/O error
        #[source]
        source: std::io::Error,
    },

    /// GPU not supported by this library
    #[error("Unsupported GPU: id=0x{id:04X}, cores={cores}")]
    UnsupportedGpu {
        /// GPU product ID
        id: u32,
        /// Number of shader cores
        cores: u32,
    },

    /// Invalid or malformed data received from driver
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Device not found or inaccessible
    #[error("GPU device not found or inaccessible")]
    DeviceNotFound,

    /// Operation not supported on this platform
    #[error("Unsupported platform (Linux/Android required)")]
    UnsupportedPlatform,

    /// Version mismatch with driver
    #[error("Driver version mismatch: required {required}, found {found}")]
    VersionMismatch {
        /// Required driver version
        required: String,
        /// Found driver version
        found: String,
    },

    /// Invalid property size encountered
    #[error("Invalid property size: {0}")]
    InvalidPropertySize(u32),

    /// Buffer too small for data
    #[error("Buffer too small: expected at least {expected} bytes, got {actual}")]
    BufferTooSmall {
        /// Minimum expected buffer size
        expected: usize,
        /// Actual buffer size
        actual: usize,
    },

    /// Driver returned invalid GPU properties
    #[error("Driver returned invalid GPU properties: {0}")]
    InvalidGpuProperties(String),

    /// CSF version check failed
    #[error("CSF version check failed: {0}")]
    CsfVersionCheck(String),
}

impl GpuError {
    /// Check if error is due to device not being found
    pub fn is_device_not_found(&self) -> bool {
        matches!(self, GpuError::DeviceNotFound)
    }

    /// Check if error is due to unsupported GPU
    pub fn is_unsupported_gpu(&self) -> bool {
        matches!(self, GpuError::UnsupportedGpu { .. })
    }

    /// Check if error is an I/O error
    pub fn is_io_error(&self) -> bool {
        matches!(self, GpuError::Io(_))
    }

    /// Check if error is an ioctl error
    pub fn is_ioctl_error(&self) -> bool {
        matches!(self, GpuError::IoctlFailed { .. })
    }

    /// Get the underlying I/O error if present
    pub fn as_io_error(&self) -> Option<&std::io::Error> {
        match self {
            GpuError::Io(e) => Some(e),
            GpuError::IoctlFailed { source, .. } => Some(source),
            _ => None,
        }
    }

    /// Check if error indicates permission issues
    pub fn is_permission_error(&self) -> bool {
        self.as_io_error()
            .map(|e| e.kind() == std::io::ErrorKind::PermissionDenied)
            .unwrap_or(false)
    }

    /// Check if error indicates the device doesn't exist
    pub fn is_not_found_error(&self) -> bool {
        self.as_io_error()
            .map(|e| e.kind() == std::io::ErrorKind::NotFound)
            .unwrap_or(false)
    }

    /// Check if error is due to invalid GPU properties
    pub fn is_invalid_properties(&self) -> bool {
        matches!(self, GpuError::InvalidGpuProperties(_))
    }
}

/// Convenience type alias for Result<T, GpuError>
pub type GpuResult<T> = Result<T, GpuError>;