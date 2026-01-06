//! IOCTL Implementierung mit Autodetection
use std::os::unix::io::RawFd;

use crate::error::{GpuError, GpuResult};

use super::ioctl::{KgslDeviceGetProperty, KgslDeviceInfo, KgslPropertyType};

/// Get KGSL device info property with autodetection
pub fn get_device_info(fd: RawFd) -> GpuResult<KgslDeviceInfo> {
    // Versuche Standard-IOCTL zuerst
    match get_device_info_standard(fd) {
        Ok(info) => Ok(info),
        Err(GpuError::IoctlFailed { .. }) | Err(GpuError::DriverNotSupported) => {
            // Fallback: Alternative IOCTLs ausprobieren
            get_device_info_alternatives(fd)
        }
        Err(e) => Err(e),
    }
}

/// Standard-IOCTL (0x80020000)
fn get_device_info_standard(fd: RawFd) -> GpuResult<KgslDeviceInfo> {
    let mut device_info = KgslDeviceInfo::default();
    
    let mut prop = KgslDeviceGetProperty {
        type_: KgslPropertyType::DeviceInfo as u32,
        value: &mut device_info as *mut _ as *mut _,
        sizebytes: std::mem::size_of::<KgslDeviceInfo>() as u32,
    };
    
    // Standard KGSL_IOCTL_GETPROPERTY = 0x80020000
    // WICHTIG: as _ lässt Rust den richtigen Typ inferieren
    const KGSL_IOCTL_GETPROPERTY: u64 = 0x80020000;
    
    unsafe {
        let result = libc::ioctl(fd, KGSL_IOCTL_GETPROPERTY as _, &mut prop);
        
        if result == 0 {
            Ok(device_info)
        } else {
            let err = std::io::Error::last_os_error();
            match err.raw_os_error() {
                Some(libc::ENOTTY) => Err(GpuError::DriverNotSupported),
                Some(libc::EINVAL) => Err(GpuError::InvalidData("Invalid argument to ioctl".into())),
                Some(libc::EPERM) | Some(libc::EACCES) => Err(GpuError::PermissionDenied),
                Some(libc::ENODEV) => Err(GpuError::DeviceNotFound),
                _ => Err(GpuError::IoctlFailed {
                    request: KGSL_IOCTL_GETPROPERTY,
                    source: err,
                }),
            }
        }
    }
}

/// Alternative IOCTLs basierend auf deinem Scan
fn get_device_info_alternatives(fd: RawFd) -> GpuResult<KgslDeviceInfo> {
    // IOCTLs die in deinem Scan funktioniert haben
    // Als u64 speichern, dann mit as _ konvertieren
    let alternative_ioctls: &[u64] = &[
        0x80006738,  // nr=0x38, size=0
        0x80006739,  // nr=0x39, size=0  
        0x8000673a,  // nr=0x3a, size=0
        0x80006740,  // nr=0x40, size=0
        0xc0006738,  // Write-Version
        0xc0006739,
        0xc000673a,
        0xc0006740,
    ];
    
    let mut last_error = None;
    
    for &ioctl_num in alternative_ioctls {
        match try_ioctl_variant(fd, ioctl_num) {
            Ok(info) => {
                // Logging für Debugging
                eprintln!("ℹ️ Using alternative ioctl: 0x{:08x}", ioctl_num);
                return Ok(info);
            }
            Err(e) => {
                last_error = Some(e);
                continue;
            }
        }
    }
    
    Err(last_error.unwrap_or(GpuError::DriverNotSupported))
}

/// Teste eine spezifische IOCTL-Variante
fn try_ioctl_variant(fd: RawFd, request: u64) -> GpuResult<KgslDeviceInfo> {
    let mut device_info = KgslDeviceInfo::default();
    
    let mut prop = KgslDeviceGetProperty {
        type_: KgslPropertyType::DeviceInfo as u32,
        value: &mut device_info as *mut _ as *mut _,
        sizebytes: std::mem::size_of::<KgslDeviceInfo>() as u32,
    };
    
    unsafe {
        // WICHTIG: as _ für platform-abhängigen Typ
        let result = libc::ioctl(fd, request as _, &mut prop);
        
        if result == 0 {
            // Überprüfe ob die Daten sinnvoll sind
            if device_info.chip_id == 0 {
                return Err(GpuError::InvalidData("Chip ID is zero".into()));
            }
            Ok(device_info)
        } else {
            let err = std::io::Error::last_os_error();
            match err.raw_os_error() {
                Some(libc::ENOTTY) => Err(GpuError::DriverNotSupported),
                Some(libc::EINVAL) => Err(GpuError::InvalidData("Invalid argument".into())),
                Some(libc::EPERM) | Some(libc::EACCES) => Err(GpuError::PermissionDenied),
                Some(libc::ENODEV) => Err(GpuError::DeviceNotFound),
                _ => Err(GpuError::IoctlFailed {
                    request,
                    source: err,
                }),
            }
        }
    }
}

/// Generic property getter (for future use)
pub fn get_property(
    fd: RawFd,
    property_type: KgslPropertyType,
    data: *mut std::ffi::c_void,
    size: usize,
) -> GpuResult<()> {
    let mut prop = KgslDeviceGetProperty {
        type_: property_type as u32,
        value: data,
        sizebytes: size as u32,
    };

    // Versuche verschiedene IOCTLs (als u64)
    let ioctls_to_try: &[u64] = &[
        0x80020000, 0x80006738, 0x80006739, 0x8000673a, 0x80006740
    ];
    
    for &request in ioctls_to_try {
        unsafe {
            // as _ für platform-abhängigen Typ
            let result = libc::ioctl(fd, request as _, &mut prop);
            
            if result == 0 {
                return Ok(());
            }
            
            // Nur bei ENOTTY weiterprobieren (andere IOCTL)
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() != Some(libc::ENOTTY) {
                return Err(GpuError::AdrenoPropertyError {
                    property: property_type as u32,
                    source: err,
                });
            }
        }
    }
    
    Err(GpuError::DriverNotSupported)
}

/// Detect which ioctl variant works on this device
pub fn detect_working_ioctl(fd: RawFd) -> GpuResult<u64> {
    let test_ioctls: &[u64] = &[
        0x80020000,  // Standard
        0x80006738,  // Alternative 1
        0x80006739,  // Alternative 2
        0x8000673a,  // Alternative 3
        0x80006740,  // Alternative 4
    ];
    
    for &request in test_ioctls {
        let mut dummy: libc::c_int = 0;
        
        unsafe {
            // as _ für platform-abhängigen Typ
            let result = libc::ioctl(fd, request as _, &mut dummy);
            
            // Auch EINVAL ist okay - bedeutet IOCTL existiert, aber Parameter falsch
            if result == 0 {
                return Ok(request);
            }
            
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::EINVAL) {
                return Ok(request);
            }
        }
    }
    
    Err(GpuError::DriverNotSupported)
}