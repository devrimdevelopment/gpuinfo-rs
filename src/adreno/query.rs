use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::path::Path;

use nix::{ioctl_readwrite, ioctl_write_ptr};

use crate::error::{GpuError, GpuResult};
use crate::info::{GpuInfo, GpuVendor, AdrenoData};

use super::database::{find_adreno_specs, SpecConfidence};
use super::ioctl::get_device_info;
use super::Mode;

/// Query Adreno GPU information with mode selection
pub fn query_adreno_with_mode<P: AsRef<Path>>(
    device_path: P,
    mode: Mode,
) -> GpuResult<GpuInfo> {
    match mode {
        Mode::Parity => query_adreno_parity(device_path),
        Mode::Extended => query_adreno_extended(device_path),
    }
}

/// Query Adreno GPU information (defaults to Parity mode)
pub fn query_adreno<P: AsRef<Path>>(device_path: P) -> GpuResult<GpuInfo> {
    query_adreno_with_mode(device_path, Mode::Parity)
}

/// Parity mode query - matches existing behavior
fn query_adreno_parity<P: AsRef<Path>>(device_path: P) -> GpuResult<GpuInfo> {
    let file = File::open(device_path).map_err(GpuError::Io)?;
    let device_info = get_device_info(file.as_raw_fd())?;
    
    // Look up specs in database
    let specs = find_adreno_specs(device_info.chip_id)
        .ok_or_else(|| GpuError::UnsupportedGpu {
            id: device_info.chip_id,
            cores: 0,
        })?;

    // Extract architecture from chip ID
    let major = ((device_info.chip_id >> 24) & 0xFF) as u8;
    let minor = ((device_info.chip_id >> 16) & 0xFF) as u8;

    let adreno_data = AdrenoData {
        chip_id: device_info.chip_id,
        gpu_model_code: device_info.gpu_model,
        mmu_enabled: device_info.mmu_enabled != 0,
        gmem_size_bytes: device_info.gmem_sizebytes,
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
        architecture_major: major,
        architecture_minor: minor,
        num_shader_cores: specs.shader_cores,
        num_l2_bytes: specs.gmem_size_kb as u64 * 1024,
        num_bus_bits: specs.bus_width_bits as u64,
        mali_data: None,
        adreno_data: Some(adreno_data),
    })
}

/// Extended mode query - with additional validation
fn query_adreno_extended<P: AsRef<Path>>(device_path: P) -> GpuResult<GpuInfo> {
    let file = File::open(device_path).map_err(GpuError::Io)?;
    let device_info = get_device_info(file.as_raw_fd())?;
    
    // Additional validation for extended mode
    if device_info.chip_id == 0 {
        return Err(GpuError::InvalidData("Chip ID is zero".into()));
    }
    
    if device_info.gmem_sizebytes == 0 {
        return Err(GpuError::InvalidData("GPU memory size is zero".into()));
    }

    // Look up specs in database
    let specs = find_adreno_specs(device_info.chip_id)
        .ok_or_else(|| GpuError::UnsupportedGpu {
            id: device_info.chip_id,
            cores: 0,
        })?;

    // Validate confidence level in extended mode
    if specs.confidence == SpecConfidence::Heuristic {
        // Use eprintln instead of log for now
        eprintln!("Warning: Using heuristic specifications for chip ID: 0x{:08x}", device_info.chip_id);
    }

    // Extract architecture from chip ID
    let major = ((device_info.chip_id >> 24) & 0xFF) as u8;
    let minor = ((device_info.chip_id >> 16) & 0xFF) as u8;

    let adreno_data = AdrenoData {
        chip_id: device_info.chip_id,
        gpu_model_code: device_info.gpu_model,
        mmu_enabled: device_info.mmu_enabled != 0,
        gmem_size_bytes: device_info.gmem_sizebytes,
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
        architecture_major: major,
        architecture_minor: minor,
        num_shader_cores: specs.shader_cores,
        num_l2_bytes: specs.gmem_size_kb as u64 * 1024,
        num_bus_bits: specs.bus_width_bits as u64,
        mali_data: None,
        adreno_data: Some(adreno_data),
    })
}