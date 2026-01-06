//! Autodetection of KGSL ioctl numbers
use std::os::unix::io::RawFd;
use crate::error::{GpuError, GpuResult};
use nix::libc;

/// Detected KGSL ioctl numbers
#[derive(Debug, Clone, Copy)]
pub struct KgslIoctls {
    pub get_property: u64,     // 0x80020000 auf normalen Geräten
    pub version: u64,          // 0x8004A001
    // ... andere die wir finden
}

impl KgslIoctls {
    /// Try to detect ioctl numbers automatically
    pub fn detect(fd: RawFd) -> GpuResult<Self> {
        let mut detected = Self::default();
        
        // Liste von bekannten/suspekten IOCTLs testen
        let candidates = [
            (0x2000, "GETPROPERTY"),  // Normal
            (0x6715, "ALTERNATIVE_1"), // Aus deinem Scan
            (0x6738, "ALTERNATIVE_2"), // Die funktionierenden
            (0x6739, "ALTERNATIVE_3"),
            (0x673a, "ALTERNATIVE_4"),
            (0x6740, "ALTERNATIVE_5"),
            (0x6741, "ALTERNATIVE_6"),
        ];
        
        for (base, name) in candidates {
            // Teste READ (0x80000000) und WRITE (0xC0000000) Varianten
            if let Ok(()) = test_ioctl(fd, 0x80000000 | (base << 2)) {
                println!("✅ Found GETPROPERTY at 0x{:08X} ({})", 0x80000000 | (base << 2), name);
                detected.get_property = 0x80000000 | (base << 2);
                break;
            }
        }
        
        if detected.get_property == 0 {
            return Err(GpuError::DriverNotSupported);
        }
        
        Ok(detected)
    }
}

impl Default for KgslIoctls {
    fn default() -> Self {
        Self {
            get_property: 0x80020000,  // Standardwert
            version: 0x8004A001,
        }
    }
}

fn test_ioctl(fd: RawFd, request: u64) -> GpuResult<()> {
    let mut dummy: libc::c_int = 0;
    
    unsafe {
        let result = libc::ioctl(fd, request as libc::c_ulong, &mut dummy);
        
        match result {
            0 => Ok(()),  // IOCTL akzeptiert (auch wenn EINVAL zurückkommt)
            _ => {
                let errno = std::io::Error::last_os_error();
                match errno.raw_os_error() {
                    Some(libc::ENOTTY) => Err(GpuError::DriverNotSupported),
                    Some(libc::EPERM) | Some(libc::EACCES) => Err(GpuError::PermissionDenied),
                    Some(libc::EINVAL) => Ok(()),  // IOCTL existiert, aber falsche Parameter
                    _ => Err(GpuError::IoctlFailed {
                        request,
                        source: errno,
                    }),
                }
            }
        }
    }
}