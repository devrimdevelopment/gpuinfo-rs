//! Simple demonstration of all query methods
use armgpuinfo::{query_gpu_auto, GpuError, GpuVendor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Simple GPU Info Demo");
    println!("====================\n");
    
    println!("1. Trying auto-detection...");
    
    let result = query_gpu_auto(None::<&str>);
    
    match result {
        Ok(info) => print_gpu_info(&info),
        Err(e) => print_error(&e),
    }
    
    println!("\n2. Library Features:");
    println!("   • Mali support: query_mali(), query_mali_with_mode()");
    println!("   • Adreno support: query_adreno()");
    println!("   • Auto-detection: query_gpu_auto()");
    
    Ok(())
}

fn print_gpu_info(info: &armgpuinfo::GpuInfo) {
    println!("✅ Found GPU:");
    println!("   Name: {}", info.gpu_name);
    println!("   Vendor: {}", info.vendor);
    println!("   Architecture: {}", info.architecture);
    println!("   Cores: {}", info.num_shader_cores);
    println!("   L2 Cache: {} KB", info.num_l2_bytes / 1024);
    
    match info.vendor {
        GpuVendor::Mali => {
            if let Some(mali) = &info.mali_data {
                println!("   GPU ID: 0x{:08X}", mali.gpu_id);
            }
            println!("   💡 Run: cargo run --example mode_demo");
        }
        GpuVendor::Adreno => {
            if let Some(adreno) = &info.adreno_data {
                println!("   Chip ID: 0x{:08X}", adreno.chip_id);
                println!("   Confidence: {}", adreno.spec_confidence);
            }
            println!("   💡 Run: cargo run --example adreno_demo");
        }
        GpuVendor::Unknown => {
            println!("   ℹ️ Unknown GPU vendor");
        }
    }
}

fn print_error(error: &GpuError) {
    match error {
        GpuError::DeviceNotFound => {
            println!("❌ No GPU device found");
            println!("   Try specifying a path manually:");
            println!("   For Mali: /dev/mali0");
            println!("   For Adreno: /dev/kgsl-3d0");
        }
        GpuError::PermissionDenied => {
            println!("❌ Permission denied");
            println!("   Try running with sudo or fix permissions");
        }
        e => {
            println!("❌ Error: {}", e);
        }
    }
}