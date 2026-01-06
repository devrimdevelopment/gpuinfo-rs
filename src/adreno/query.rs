use std::borrow::Cow;
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::path::Path;

use crate::error::{GpuError, GpuResult};
use crate::info::{GpuInfo, GpuVendor, AdrenoData};

use super::database::{find_adreno_specs, SpecConfidence};
use super::ioctl_impl::{get_device_info, detect_working_ioctl};
use super::ioctl::KgslDeviceInfo;  // Typ aus ioctl.rs
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

/// Common function to create GpuInfo from device info and specs
fn create_gpu_info_from_specs(
    device_info: &KgslDeviceInfo,
    specs: &super::database::AdrenoSpecs,
) -> GpuInfo {
    // Extract architecture from chip ID
    let major = ((device_info.chip_id >> 24) & 0xFF) as u8;
    let minor = ((device_info.chip_id >> 16) & 0xFF) as u8;

    let adreno_data = AdrenoData {
        chip_id: device_info.chip_id,
        gpu_model_code: device_info.gpu_model,
        mmu_enabled: device_info.mmu_enabled != 0,
        gmem_size_bytes: device_info.gmem_sizebytes,
        spec_confidence: specs.confidence.as_cow(),
        stream_processors: specs.stream_processors,
        max_freq_mhz: specs.max_freq_mhz,
        process_nm: specs.process_nm,
        release_year: specs.year,
        snapdragon_models: specs.snapdragon_models
            .iter()
            .map(|&s| Cow::Borrowed(s))
            .collect(),
    };

    GpuInfo {
        vendor: GpuVendor::Adreno,
        gpu_name: Cow::Borrowed(specs.name),
        architecture: specs.architecture.to_string().into(),
        architecture_major: major,
        architecture_minor: minor,
        num_shader_cores: specs.shader_cores,
        num_l2_bytes: specs.gmem_size_kb as u64 * 1024,
        num_bus_bits: specs.bus_width_bits as u64,
        mali_data: None,
        adreno_data: Some(adreno_data),
    }
}

/// Parity mode query - matches existing behavior
fn query_adreno_parity<P: AsRef<Path>>(device_path: P) -> GpuResult<GpuInfo> {
    let file = match File::open(&device_path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(GpuError::DeviceNotFound);
        }
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            return Err(GpuError::PermissionDenied);
        }
        Err(e) => return Err(GpuError::Io(e)),
    };
    
    let fd = file.as_raw_fd();
    
    // Debug: Try to detect which ioctl works
    #[cfg(debug_assertions)]
    match detect_working_ioctl(fd) {
        Ok(ioctl_num) => eprintln!("🔍 Detected working ioctl: 0x{:08x}", ioctl_num),
        Err(e) => eprintln!("⚠️ Could not detect ioctl: {}", e),
    }
    
    let device_info = get_device_info(fd)?;
    
    // Validate basic device info
    if device_info.chip_id == 0 {
        return Err(GpuError::InvalidData("Chip ID is zero".into()));
    }
    
    // Look up specs in database
    let specs = find_adreno_specs(device_info.chip_id)
        .ok_or_else(|| GpuError::UnsupportedGpu {
            id: device_info.chip_id,
            cores: 0,
        })?;

    Ok(create_gpu_info_from_specs(&device_info, specs))
}

/// Extended mode query - with additional validation
fn query_adreno_extended<P: AsRef<Path>>(device_path: P) -> GpuResult<GpuInfo> {
    let file = match File::open(&device_path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(GpuError::DeviceNotFound);
        }
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            return Err(GpuError::PermissionDenied);
        }
        Err(e) => return Err(GpuError::Io(e)),
    };
    
    let fd = file.as_raw_fd();
    
    let device_info = get_device_info(fd)?;
    
    // Extended validation
    if device_info.chip_id == 0 {
        return Err(GpuError::InvalidData("Chip ID is zero".into()));
    }
    
    if device_info.gmem_sizebytes == 0 {
        return Err(GpuError::InvalidData("GPU memory size is zero".into()));
    }
    
    if device_info.device_id == 0 {
        eprintln!("⚠️ Device ID is zero, might be incomplete driver info");
    }

    // Look up specs in database
    let specs = find_adreno_specs(device_info.chip_id)
        .ok_or_else(|| GpuError::UnsupportedGpu {
            id: device_info.chip_id,
            cores: 0,
        })?;

    // Validate confidence level in extended mode
    if specs.confidence == SpecConfidence::Heuristic {
        eprintln!("⚠️ Using heuristic specifications for chip ID: 0x{:08x}", device_info.chip_id);
    }

    let info = create_gpu_info_from_specs(&device_info, specs);
    
    // Additional validation for extended mode
    validate_extended_info(&info)?;
    
    Ok(info)
}

/// Validate GPU info for extended mode
fn validate_extended_info(info: &GpuInfo) -> GpuResult<()> {
    if info.num_shader_cores == 0 {
        return Err(GpuError::InvalidData("Shader core count is zero".into()));
    }
    
    if info.num_l2_bytes == 0 {
        return Err(GpuError::InvalidData("L2 cache size is zero".into()));
    }
    
    // Validate architecture version makes sense
    if info.architecture_major < 4 || info.architecture_major > 9 {
        return Err(GpuError::InvalidData(format!(
            "Invalid architecture major version: {}",
            info.architecture_major
        )));
    }
    
    Ok(())
}

/// Type alias for query functions
type QueryFn<P> = fn(P) -> GpuResult<GpuInfo>;

/// Try multiple methods to query Adreno GPU
pub fn query_adreno_robust<P: AsRef<Path>>(device_path: P) -> GpuResult<GpuInfo> {
    // Explizite Funktionszeiger-Typen
    let methods: &[(&str, fn(&Path) -> GpuResult<GpuInfo>)] = &[
        ("Extended mode", query_adreno_extended_ref as fn(&Path) -> GpuResult<GpuInfo>),
        ("Parity mode", query_adreno_parity_ref as fn(&Path) -> GpuResult<GpuInfo>),
    ];
    
    let mut last_error = None;
    let mut tried_methods = Vec::new();
    
    for (name, method) in methods {
        tried_methods.push(*name);
        match method(device_path.as_ref()) {
            Ok(info) => {
                if tried_methods.len() > 1 {
                    eprintln!("✅ Success with {} after trying: {}", name, tried_methods.join(" → "));
                }
                return Ok(info);
            }
            Err(e) => {
                eprintln!("❌ {} failed: {}", name, e);
                last_error = Some(e);
            }
        }
    }
    
    Err(last_error.unwrap_or(GpuError::DeviceNotFound))
}

// Hilfsfunktionen mit &Path statt generischem P
fn query_adreno_extended_ref(device_path: &Path) -> GpuResult<GpuInfo> {
    query_adreno_extended(device_path)
}

fn query_adreno_parity_ref(device_path: &Path) -> GpuResult<GpuInfo> {
    query_adreno_parity(device_path)
}
/// Debug function to print detailed device info
#[cfg(feature = "debug")]
pub fn debug_device_info<P: AsRef<Path>>(device_path: P) -> GpuResult<()> {
    use super::ioctl::KgslDeviceInfo;
    
    let file = File::open(device_path).map_err(GpuError::Io)?;
    let fd = file.as_raw_fd();
    
    println!("🔍 Debug KGSL Device Info");
    println!("=========================");
    
    // Try to detect working ioctl
    match detect_working_ioctl(fd) {
        Ok(ioctl_num) => println!("Working ioctl: 0x{:08x}", ioctl_num),
        Err(e) => println!("Could not detect ioctl: {}", e),
    }
    
    // Try to get device info
    match get_device_info(fd) {
        Ok(info) => {
            println!("Device Info:");
            println!("  Device ID: 0x{:08x}", info.device_id);
            println!("  Chip ID:   0x{:08x}", info.chip_id);
            println!("  MMU:       {}", if info.mmu_enabled != 0 { "enabled" } else { "disabled" });
            println!("  GMEM Base: 0x{:08x}", info.gmem_gpubaseaddr);
            println!("  GMEM Size: {} bytes ({} KB)", 
                info.gmem_sizebytes, info.gmem_sizebytes / 1024);
            println!("  GPU Model: 0x{:08x}", info.gpu_model);
            println!("  Unknown1:  0x{:08x}", info.unknown1);
            println!("  Unknown2:  0x{:08x}", info.unknown2);
            
            // Try to find in database
            if let Some(specs) = find_adreno_specs(info.chip_id) {
                println!("\nDatabase Match:");
                println!("  Name:      {}", specs.name);
                println!("  Arch:      {}", specs.architecture);
                println!("  Cores:     {}", specs.shader_cores);
                println!("  Confidence: {}", specs.confidence);
            } else {
                println!("\n❌ No database entry for chip ID: 0x{:08x}", info.chip_id);
                
                // Show architecture bits
                let major = (info.chip_id >> 24) & 0xFF;
                let minor = (info.chip_id >> 16) & 0xFF;
                let gen = (info.chip_id >> 8) & 0xFF;
                let rev = info.chip_id & 0xFF;
                
                println!("  Architecture bits:");
                println!("    Major:    0x{:02x} ({})", major, major);
                println!("    Minor:    0x{:02x} ({})", minor, minor);
                println!("    Gen:      0x{:02x} ({})", gen, gen);
                println!("    Rev:      0x{:02x} ({})", rev, rev);
            }
        }
        Err(e) => {
            println!("❌ Failed to get device info: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}