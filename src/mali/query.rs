
use std::fs::OpenOptions;
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::Path;

use nix::{ioctl_readwrite, ioctl_write_ptr};

use crate::error::{GpuError, GpuResult};
use crate::info::{GpuInfo, GpuVendor, MaliData};
use crate::Mode;

use super::parser::{parse_properties, parse_properties_lenient, ParserConfig};
use super::database::{get_gpu_id, lookup_product, extract_architecture, validate_gpu_info};

// Constants
const MALI_IOC_MAGIC: u8 = 0x80;

// Ioctl request numbers
mod ioctl_num {
    pub const SET_FLAGS: u64 = 0x01;
    pub const GET_PROPS: u64 = 0x03;
    pub const VERSION_CHECK_CSF: u64 = 0x34;
}

// Ioctl structures
#[repr(C)]
struct VersionCheck {
    major: u16,
    minor: u16,
}

#[repr(C)]
struct SetFlags {
    create_flags: u32,
}

#[repr(C)]
struct MaliPropsQuery {
    buffer: u64,
    size: u32,
    flags: u32,
}

ioctl_readwrite!(mali_version_check_csf, MALI_IOC_MAGIC, 0x34, VersionCheck);
ioctl_write_ptr!(mali_set_flags, MALI_IOC_MAGIC, 0x01, SetFlags);
ioctl_write_ptr!(mali_get_props, MALI_IOC_MAGIC, 0x03, MaliPropsQuery);

/// Query Mali GPU information with mode selection
pub fn query_mali_with_mode<P: AsRef<Path>>(device_path: P, mode: Mode) -> GpuResult<GpuInfo> {
    match mode {
        Mode::Parity => ParityStrategy.query(device_path),
        Mode::Extended => ExtendedStrategy.query(device_path),
    }
}

/// Query Mali GPU information (defaults to Parity mode)
pub fn query_mali<P: AsRef<Path>>(device_path: P) -> GpuResult<GpuInfo> {
    query_mali_with_mode(device_path, Mode::Parity)
}

/// Trait defining the strategy for querying Mali GPU information
trait QueryStrategy {
    fn query<P: AsRef<Path>>(&self, device_path: P) -> GpuResult<GpuInfo>;
    fn parser_config(&self) -> ParserConfig;
    fn get_properties(&self, fd: RawFd) -> GpuResult<Vec<u8>>;
    fn should_validate(&self) -> bool;
    fn use_product_db(&self) -> bool;
}

/// Parity strategy - minimal like libgpuinfo
struct ParityStrategy;

impl QueryStrategy for ParityStrategy {
    fn query<P: AsRef<Path>>(&self, device_path: P) -> GpuResult<GpuInfo> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(device_path)
            .map_err(GpuError::Io)?;

        let fd = file.as_raw_fd();
        let props = self.get_properties(fd)?;
        let parsed = parse_properties_lenient(&props);

        let num_l2_bytes = if parsed.l2_log2_cache_size > 0 && parsed.num_l2_slices > 0 {
            (1u64 << parsed.l2_log2_cache_size) * parsed.num_l2_slices
        } else {
            0
        };

        // Try to get product info from database
        let (gpu_name, architecture, arch_major, arch_minor, gpu_id) =
            if self.use_product_db() {
                if let Some(product_info) = lookup_product(get_gpu_id(parsed.gpu_id), parsed.num_shader_cores) {
                    let (major, minor) = extract_architecture(parsed.raw_gpu_id);
                    (
                        product_info.name.to_string(),
                        product_info.architecture.to_string(),
                        major,
                        minor,
                        get_gpu_id(parsed.gpu_id)
                    )
                } else {
                    (String::new(), String::new(), 0, 0, parsed.gpu_id)
                }
            } else {
                (String::new(), String::new(), 0, 0, parsed.gpu_id)
            };

        let mali_data = MaliData {
            gpu_id: parsed.gpu_id,
            raw_gpu_id: parsed.raw_gpu_id,
            shader_core_mask: parsed.shader_core_mask,
            num_l2_slices: parsed.num_l2_slices,
            num_exec_engines: 0,
            num_fp32_fmas_per_core: 0,
            num_fp16_fmas_per_core: 0,
            num_texels_per_core: 0,
            num_pixels_per_core: 0,
        };

        Ok(GpuInfo {
            vendor: GpuVendor::Mali,
            gpu_name,
            architecture,
            architecture_major: arch_major,
            architecture_minor: arch_minor,
            num_shader_cores: parsed.num_shader_cores,
            num_l2_bytes,
            num_bus_bits: 0,
            mali_data: Some(mali_data),
            adreno_data: None,
        })
    }

    fn parser_config(&self) -> ParserConfig {
        ParserConfig::PARITY
    }

    fn get_properties(&self, fd: RawFd) -> GpuResult<Vec<u8>> {
        get_properties_common(fd)
    }

    fn should_validate(&self) -> bool {
        false
    }

    fn use_product_db(&self) -> bool {
        true
    }
}

/// Extended strategy - full features
struct ExtendedStrategy;

impl QueryStrategy for ExtendedStrategy {
    fn query<P: AsRef<Path>>(&self, device_path: P) -> GpuResult<GpuInfo> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(device_path)
            .map_err(GpuError::Io)?;

        let fd = file.as_raw_fd();

        // Check version (ignore errors)
        let _ = check_version_optional(fd);

        // Set flags (ignore errors)
        let _ = set_flags_optional(fd);

        // Get properties
        let props = self.get_properties(fd)?;
        let parsed = parse_properties(&props, self.parser_config())?;

        let product_info = lookup_product(get_gpu_id(parsed.gpu_id), parsed.num_shader_cores)
            .ok_or_else(|| GpuError::UnsupportedGpu {
                id: parsed.gpu_id,
                cores: parsed.num_shader_cores,
            })?;

        let num_exec_engines = (product_info.get_num_exec_engines)(
            parsed.num_shader_cores,
            parsed.raw_core_features,
            parsed.raw_thread_features,
        );

        let num_fp32_fmas_per_engine = (product_info.get_num_fp32_fmas_per_engine)(
            parsed.num_shader_cores,
            parsed.raw_core_features,
            parsed.raw_thread_features,
        );

        let num_fp32_fmas_per_core = num_fp32_fmas_per_engine * num_exec_engines;

        let num_texels_per_core = (product_info.get_num_texels)(
            parsed.num_shader_cores,
            parsed.raw_core_features,
            parsed.raw_thread_features,
        );

        let num_pixels_per_core = (product_info.get_num_pixels)(
            parsed.num_shader_cores,
            parsed.raw_core_features,
            parsed.raw_thread_features,
        );

        let (arch_major, arch_minor) = extract_architecture(parsed.raw_gpu_id);

        let num_l2_bytes = (1u64 << parsed.l2_log2_cache_size) * parsed.num_l2_slices;
        let num_bus_bits = 1u64 << ((parsed.raw_l2_features >> 24) & 0xFF);

        let mali_data = MaliData {
            gpu_id: get_gpu_id(parsed.gpu_id),
            raw_gpu_id: parsed.raw_gpu_id,
            shader_core_mask: parsed.shader_core_mask,
            num_l2_slices: parsed.num_l2_slices,
            num_exec_engines,
            num_fp32_fmas_per_core,
            num_fp16_fmas_per_core: num_fp32_fmas_per_core * 2,
            num_texels_per_core,
            num_pixels_per_core,
        };

        let info = GpuInfo {
            vendor: GpuVendor::Mali,
            gpu_name: product_info.name.to_string(),
            architecture: product_info.architecture.to_string(),
            architecture_major: arch_major,
            architecture_minor: arch_minor,
            num_shader_cores: parsed.num_shader_cores,
            num_l2_bytes,
            num_bus_bits,
            mali_data: Some(mali_data),
            adreno_data: None,
        };

        if self.should_validate() {
            validate_gpu_info(&info)?;
        }

        Ok(info)
    }

    fn parser_config(&self) -> ParserConfig {
        ParserConfig::EXTENDED
    }

    fn get_properties(&self, fd: RawFd) -> GpuResult<Vec<u8>> {
        get_properties_common(fd)
    }

    fn should_validate(&self) -> bool {
        true
    }

    fn use_product_db(&self) -> bool {
        true
    }
}

/// Common function to get properties
fn get_properties_common(fd: RawFd) -> GpuResult<Vec<u8>> {
    let mut query = MaliPropsQuery {
        buffer: 0,
        size: 0,
        flags: 0,
    };

    let needed_size = unsafe {
        mali_get_props(fd, &mut query).map_err(|e| GpuError::IoctlFailed {
            request: ioctl_num::GET_PROPS,
            source: e.into(),
        })?
    } as usize;

    if needed_size == 0 {
        return Err(GpuError::InvalidData("Driver returned zero buffer size".into()));
    }

    let mut buffer = vec![0u8; needed_size];
    query.buffer = buffer.as_mut_ptr() as u64;
    query.size = needed_size as u32;

    unsafe {
        mali_get_props(fd, &mut query).map_err(|e| GpuError::IoctlFailed {
            request: ioctl_num::GET_PROPS,
            source: e.into(),
        })?;
    }

    Ok(buffer)
}

/// Optional version check (errors ignored)
fn check_version_optional(fd: RawFd) -> GpuResult<()> {
    let mut ver = VersionCheck { major: 0, minor: 0 };
    match unsafe { mali_version_check_csf(fd, &mut ver) } {
        Ok(_) => Ok(()),
        Err(nix::Error::EACCES) | Err(nix::Error::EPERM) | Err(nix::Error::ENOTTY) => {
            // Permission denied or not supported - that's okay
            Ok(())
        }
        Err(e) => Err(GpuError::IoctlFailed {
            request: ioctl_num::VERSION_CHECK_CSF,
            source: e.into(),
        }),
    }
}

/// Optional set flags (errors ignored)
fn set_flags_optional(fd: RawFd) -> GpuResult<()> {
    let flags = SetFlags { create_flags: 2 };
    match unsafe { mali_set_flags(fd, &flags) } {
        Ok(_) => Ok(()),
        Err(nix::Error::EACCES) | Err(nix::Error::EPERM) | Err(nix::Error::ENOTTY) => {
            // Permission denied or not supported - that's okay
            Ok(())
        }
        Err(e) => Err(GpuError::IoctlFailed {
            request: ioctl_num::SET_FLAGS,
            source: e.into(),
        }),
    }
}