//! Demonstrate Adreno GPU query functionality
//! Shows both Parity and Extended modes

use armgpuinfo::adreno::{query_adreno, query_adreno_with_mode, Mode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Adreno GPU Query Demo");
    println!("=====================\n");
    
    // Common Adreno device paths
    let device_paths = [
        "/dev/kgsl-3d0",      // Standard KGSL path
        "/dev/kgsl/kgsl-3d0", // Alternative path
    ];
    
    // Find first accessible device
    let device_path = device_paths
        .iter()
        .find(|&&path| std::path::Path::new(path).exists())
        .unwrap_or(&device_paths[0]);
    
    println!("Using device: {}", device_path);
    
    // 1. Default query (Parity mode)
    println!("\n1. Default Query (Parity Mode):");
    match query_adreno(device_path) {
        Ok(info) => {
            print_adreno_info(&info, false);
        }
        Err(e) => {
            handle_adreno_error(&e, device_path);
            
            // Try with different mode if driver not supported
            if e.is_driver_not_supported() {
                println!("\n💡 Trying alternative approach...");
            }
        }
    }
    
    // 2. Extended mode query
    println!("\n2. Extended Mode Query:");
    match query_adreno_with_mode(device_path, Mode::Extended) {
        Ok(info) => {
            print_adreno_info(&info, true);
            
            // Show confidence warning if needed
            if let Some(adreno) = &info.adreno_data {
                if adreno.spec_confidence.contains("Heuristic") {
                    println!("\n⚠️  Warning: Using heuristic specifications");
                    println!("   Some details may be estimated");
                }
            }
        }
        Err(e) => {
            println!("   ❌ Error: {}", e);
            
            // Extended mode is stricter - this might be expected
            if let armgpuinfo::GpuError::InvalidData(msg) = &e {
                println!("   💡 Extended mode validation: {}", msg);
            }
        }
    }
    
    // 3. Show available information about modes
    println!("\n3. Query Mode Information:");
    println!("   • Parity Mode: Fast, lenient, compatible");
    println!("   • Extended Mode: Validated, detailed, strict");
    println!("   • Use Parity for broad compatibility");
    println!("   • Use Extended when you need reliability");
    
    Ok(())
}

fn print_adreno_info(info: &armgpuinfo::GpuInfo, extended: bool) {
    println!("   ✅ Success!");
    println!("   Vendor: {:?}", info.vendor);
    println!("   Model: {}", info.gpu_name);
    println!("   Architecture: {}", info.architecture);
    println!("   Shader Cores: {}", info.num_shader_cores);
    
    if let Some(adreno) = &info.adreno_data {
        println!("   Chip ID: 0x{:08X}", adreno.chip_id);
        println!("   GMEM Size: {} MB", adreno.gmem_size_bytes / 1024 / 1024);
        println!("   Confidence: {}", adreno.spec_confidence);
        
        if extended {
            println!("   Stream Processors: {}", adreno.stream_processors);
            println!("   Max Frequency: {} MHz", adreno.max_freq_mhz);
            println!("   Process: {} nm", adreno.process_nm);
            println!("   Year: {}", adreno.release_year);
            
            if !adreno.snapdragon_models.is_empty() {
                println!("   Snapdragon Models: {}", adreno.snapdragon_models.join(", "));
            }
        }
        
        // Show architecture details
        let major = adreno.chip_id >> 24 & 0xFF;
        let minor = adreno.chip_id >> 16 & 0xFF;
        println!("   Architecture: {}.{}.x.x", major, minor);
    }
    
    println!("   L2 Cache: {} KB", info.num_l2_bytes / 1024);
    println!("   Bus Width: {} bits", info.num_bus_bits);
}

fn handle_adreno_error(e: &armgpuinfo::GpuError, device_path: &str) {
    println!("   ❌ Error: {}", e);
    
    match e {
        armgpuinfo::GpuError::DeviceNotFound => {
            println!("   💡 Device not found: {}", device_path);
            println!("   Possible fixes:");
            println!("     1. Check if device exists: ls {}", device_path);
            println!("     2. Check permissions: ls -la {}", device_path);
            println!("     3. Load GPU driver if needed");
        }
        armgpuinfo::GpuError::PermissionDenied => {
            println!("   💡 Permission denied");
            println!("   Try: sudo chmod 666 {}", device_path);
            println!("   Or run with sudo");
        }
        armgpuinfo::GpuError::DriverNotSupported => {
            println!("   💡 KGSL driver not supported");
            println!("   Make sure Qualcomm Adreno driver is loaded");
        }
        armgpuinfo::GpuError::UnsupportedGpu { id, cores: _ } => {
            println!("   💡 Unsupported GPU: 0x{:08X}", id);
            println!("   Please report this chip ID");
        }
        _ => {}
    }
}