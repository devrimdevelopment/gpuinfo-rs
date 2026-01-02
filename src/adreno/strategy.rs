//! Strategy Pattern for Adreno GPU Queries
//! Consistent with Mali implementation

use std::fs::File;
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::Path;

use crate::error::{GpuError, GpuResult};
use crate::info::{GpuInfo, GpuVendor, AdrenoData};

use super::database::{find_adreno_specs, ConfidenceLevel};
use super::ioctl::{get_device_info, optional_ioctl};
use super::parser::{parse_device_info_lenient, parse_device_info_strict, ParserConfig};

/// Trait defining the strategy for querying Adreno GPU information
pub trait AdrenoQueryStrategy {
    fn query<P: AsRef<Path>>(&self, device_path: P) -> GpuResult<GpuInfo>;
    fn parser_config(&self) -> ParserConfig;
    fn get_device_info_bytes(&self, fd: RawFd) -> GpuResult<Vec<u8>>;
    fn should_validate(&self) -> bool;
    fn use_extended_database(&self) -> bool;
    fn extract_architecture(&self, chip_id: u32) -> (u8, u8);
}

/// Parity strategy - minimal like existing behavior
pub struct ParityStrategy;

impl AdrenoQueryStrategy for ParityStrategy {
    fn query<P: AsRef<Path>>(&self, device_path: P) -> GpuResult<GpuInfo> {
        let file = File::open(device_path).map_err(GpuError::Io)?;
        let fd = file.as_raw_fd();
        
        // Get raw property bytes
        let info_bytes = self.get_device_info_bytes(fd)?;
        
        // Parse leniently (like current behavior)
        let parsed_info = parse_device_info_lenient(&info_bytes);
        
        // Look up specs in database
        let specs = find_adreno_specs(parsed_info.chip_id)
            .ok_or_else(|| GpuError::UnsupportedGpu {
                id: parsed_info.chip_id,
                cores: 0,
            })?;

        // Extract architecture
        let (arch_major, arch_minor) = self.extract_architecture(parsed_info.chip_id);

        let adreno_data = AdrenoData {
            chip_id: parsed_info.chip_id,
            gpu_model_code: parsed_info.gpu_model,
            mmu_enabled: parsed_info.mmu_enabled,
            gmem_size_bytes: parsed_info.gmem_sizebytes,
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
            architecture_major: arch_major,
            architecture_minor: arch_minor,
            num_shader_cores: specs.shader_cores,
            num_l2_bytes: specs.gmem_size_kb as u64 * 1024,
            num_bus_bits: specs.bus_width_bits as u64,
            mali_data: None,
            adreno_data: Some(adreno_data),
        })
    }

    fn parser_config(&self) -> ParserConfig {
        ParserConfig::PARITY
    }

    fn get_device_info_bytes(&self, fd: RawFd) -> GpuResult<Vec<u8>> {
        // Use the typed ioctl wrapper
        let device_info = get_device_info(fd).map_err(|e| GpuError::IoctlFailed {
            request: 0x80020000, // KGSL_IOCTL_GETPROPERTY
            source: e.into(),
        })?;
        
        // Convert to bytes for parser
        let bytes = unsafe {
            std::slice::from_raw_parts(
                &device_info as *const _ as *const u8,
                std::mem::size_of_val(&device_info)
            )
        };
        
        Ok(bytes.to_vec())
    }

    fn should_validate(&self) -> bool {
        false
    }

    fn use_extended_database(&self) -> bool {
        true
    }

    fn extract_architecture(&self, chip_id: u32) -> (u8, u8) {
        let major = ((chip_id >> 24) & 0xFF) as u8;
        let minor = ((chip_id >> 16) & 0xFF) as u8;
        (major, minor)
    }
}

/// Extended strategy - full features with validation
pub struct ExtendedStrategy;

impl AdrenoQueryStrategy for ExtendedStrategy {
    fn query<P: AsRef<Path>>(&self, device_path: P) -> GpuResult<GpuInfo> {
        let file = File::open(device_path).map_err(GpuError::Io)?;
        let fd = file.as_raw_fd();

        // Optional ioctls (like version checks if they exist)
        let _ = optional_ioctl(fd);

        // Get and parse device info with strict validation
        let info_bytes = self.get_device_info_bytes(fd)?;
        let parsed_info = parse_device_info_strict(&info_bytes)?;

        // Look up specs with extended database
        let specs = find_adreno_specs(parsed_info.chip_id)
            .ok_or_else(|| GpuError::UnsupportedGpu {
                id: parsed_info.chip_id,
                cores: 0,
            })?;

        // Validate confidence level if needed
        if self.should_validate() && specs.confidence == ConfidenceLevel::Estimated {
            return Err(GpuError::InsufficientData {
                chip_id: parsed_info.chip_id,
                details: "Only estimated specs available".into(),
            });
        }

        let (arch_major, arch_minor) = self.extract_architecture(parsed_info.chip_id);

        let adreno_data = AdrenoData {
            chip_id: parsed_info.chip_id,
            gpu_model_code: parsed_info.gpu_model,
            mmu_enabled: parsed_info.mmu_enabled,
            gmem_size_bytes: parsed_info.gmem_sizebytes,
            spec_confidence: specs.confidence.to_string(),
            stream_processors: specs.stream_processors,
            max_freq_mhz: specs.max_freq_mhz,
            process_nm: specs.process_nm,
            release_year: specs.year,
            snapdragon_models: specs.snapdragon_models.iter().map(|&s| s.to_string()).collect(),
        };

        let info = GpuInfo {
            vendor: GpuVendor::Adreno,
            gpu_name: specs.name.to_string(),
            architecture: specs.architecture.to_string(),
            architecture_major: arch_major,
            architecture_minor: arch_minor,
            num_shader_cores: specs.shader_cores,
            num_l2_bytes: specs.gmem_size_kb as u64 * 1024,
            num_bus_bits: specs.bus_width_bits as u64,
            mali_data: None,
            adreno_data: Some(adreno_data),
        };

        // Additional validation if configured
        if self.should_validate() {
            // Could add custom validation here
        }

        Ok(info)
    }

    fn parser_config(&self) -> ParserConfig {
        ParserConfig::EXTENDED
    }

    fn get_device_info_bytes(&self, fd: RawFd) -> GpuResult<Vec<u8>> {
        ParityStrategy.get_device_info_bytes(fd)
    }

    fn should_validate(&self) -> bool {
        true
    }

    fn use_extended_database(&self) -> bool {
        true
    }

    fn extract_architecture(&self, chip_id: u32) -> (u8, u8) {
        let major = ((chip_id >> 24) & 0xFF) as u8;
        let minor = ((chip_id >> 16) & 0xFF) as u8;
        (major, minor)
    }
}