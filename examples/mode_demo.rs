//! Demonstrate different query modes for Mali GPUs
//! Shows Parity vs Extended mode differences

use armgpuinfo::{query_gpu_with_mode, Mode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Mali GPU Mode Comparison Demo");
    println!("=============================\n");
    
    // Device path (adjust if needed)
    let device_path = "/dev/mali0";
    
    // Test Parity Mode (lenient, like libgpuinfo)
    println!("1. Parity Mode (lenient):");
    match query_gpu_with_mode(device_path, Mode::Parity) {
        Ok(info) => {
            println!("   ✅ Success!");
            print_gpu_info(&info, false);
        }
        Err(e) => {
            println!("   ❌ Error: {}", e);
            if e.is_permission_error() {
                println!("   💡 Try: sudo chmod 666 {}", device_path);
            }
        }
    }
    
    println!("\n2. Extended Mode (strict validation):");
    match query_gpu_with_mode(device_path, Mode::Extended) {
        Ok(info) => {
            println!("   ✅ Success!");
            print_gpu_info(&info, true);
        }
        Err(e) => {
            println!("   ❌ Error: {}", e);
            println!("   💡 Extended mode enforces stricter validation");
        }
    }
    
    println!("\n3. Mode Differences:");
    println!("   • Parity:   Lenient, ignores some errors");
    println!("   • Extended: Strict validation, more details");
    println!("   • Use Parity for compatibility");
    println!("   • Use Extended for reliability");
    
    Ok(())
}

fn print_gpu_info(info: &armgpuinfo::GpuInfo, extended: bool) {
    println!("   Vendor: {:?}", info.vendor);
    println!("   Model: {}", info.gpu_name);
    println!("   Architecture: {}", info.architecture);
    println!("   Shader Cores: {}", info.num_shader_cores);
    
    if let Some(mali) = &info.mali_data {
        println!("   GPU ID: 0x{:08X}", mali.gpu_id);
        println!("   L2 Slices: {}", mali.num_l2_slices);
        
        if extended {
            println!("   Execution Engines: {}", mali.num_exec_engines);
            println!("   FP32 FMAs/Core: {}", mali.num_fp32_fmas_per_core);
            println!("   Texels/Core: {}", mali.num_texels_per_core);
        }
    }
    
    println!("   L2 Cache: {} bytes", info.num_l2_bytes);
    println!("   Bus Width: {} bits", info.num_bus_bits);
}