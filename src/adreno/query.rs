use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::path::Path;

use libc;

use crate::error::{GpuError, GpuResult};
use crate::info::{GpuInfo, GpuVendor, AdrenoData};

#[allow(unused_imports)]
use super::database::{AdrenoSpecs, find_adreno_specs};

const KGSL_IOC_TYPE: u8 = b'p';

// ioctl macro for KGSL
macro_rules! iorw {
    ($g:expr, $n:expr, $t:expr) => {
        (($g as u32) << 8) | (($n as u32) & 0xFF) | ((($t as u32) & 0x3FFF) << 16) | 0xC0000000
    };
}

#[repr(C)]
struct KgslDeviceGetProperty {
    type_: u32,
    value: *mut std::ffi::c_void,
    sizebytes: u32,
}

// Real structure based on the bytes
#[repr(C)]
#[derive(Debug)]
struct KgslDeviceInfo {
    pub device_id: u32,
    pub chip_id: u32,
    pub mmu_enabled: u32,
    pub gmem_gpubaseaddr: u32,
    pub gmem_sizebytes: u32,
    pub unknown1: u32,
    pub unknown2: u32,
    pub gpu_model: u32,
}

unsafe fn ioctl_kgsl_getproperty(fd: i32, arg: &mut KgslDeviceGetProperty) -> i32 {
    libc::ioctl(
        fd,
        iorw!(KGSL_IOC_TYPE, 0x02, std::mem::size_of::<KgslDeviceGetProperty>()) as i32 as libc::c_ulong,
        arg
    )
}

/// Query Adreno GPU information
pub fn query_adreno<P: AsRef<Path>>(device_path: P) -> GpuResult<GpuInfo> {
    let file = File::open(device_path).map_err(GpuError::Io)?;

    // Query DEVICE_INFO
    let mut dev_info: KgslDeviceInfo = unsafe { std::mem::zeroed() };
    let mut get_prop = KgslDeviceGetProperty {
        type_: 0x1,  // KGSL_PROP_DEVICE_INFO
        value: &mut dev_info as *mut _ as *mut std::ffi::c_void,
        sizebytes: std::mem::size_of::<KgslDeviceInfo>() as u32,
    };

    let result = unsafe { ioctl_kgsl_getproperty(file.as_raw_fd(), &mut get_prop) };

    if result < 0 {
        return Err(GpuError::IoctlFailed {
            request: 0x80020000, // KGSL_IOCTL_GETPROPERTY
            source: std::io::Error::last_os_error(),
        });
    }

    // Look up specs in database
    let specs = find_adreno_specs(dev_info.chip_id)
        .ok_or_else(|| GpuError::UnsupportedGpu {
            id: dev_info.chip_id,
            cores: 0,
        })?;

    // Extract architecture from chip ID
    let major = (dev_info.chip_id >> 24) & 0xFF;
    let minor = (dev_info.chip_id >> 16) & 0xFF;

    let adreno_data = AdrenoData {
        chip_id: dev_info.chip_id,
        gpu_model_code: dev_info.gpu_model,
        mmu_enabled: dev_info.mmu_enabled != 0,
        gmem_size_bytes: dev_info.gmem_sizebytes,
        spec_confidence: specs.confidence.to_string(),
        stream_processors: specs.stream_processors,
        max_freq_mhz: specs.max_freq_mhz,
        process_nm: specs.process_nm,
        release_year: specs.year,
        snapdragon_models: specs.snapdragon_models.iter().map(|&s| s.to_string()).collect(),
    };

    Ok(GpuInfo {
        vendor: GpuVendor::Adreno,
        gpu_name: specs.name.to_string(),
        architecture: specs.architecture.to_string(),
        architecture_major: major as u8,
        architecture_minor: minor as u8,
        num_shader_cores: specs.shader_cores,
        num_l2_bytes: specs.gmem_size_kb as u64 * 1024,
        num_bus_bits: specs.bus_width_bits as u64,
        mali_data: None,
        adreno_data: Some(adreno_data),
    })
}