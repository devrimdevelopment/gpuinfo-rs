//! Demonstration of automatic GPU detection

use gpu_info::{query_gpu_auto, GpuResult};

fn main() -> GpuResult<()> {
    println!("Auto GPU Detection Demo");
    println!("=======================\n");

    // Try auto-detection without specifying device path
    match query_gpu_auto(None::<&str>) {
        Ok(info) => {
            println!("✅ Auto-detected GPU:");
            println!("  Vendor:        {}", info.vendor);
            println!("  Name:          {}", info.gpu_name);
            println!("  Architecture:  {} ({}.{})",
                info.architecture, info.architecture_major, info.architecture_minor);
            println!("  Shader Cores:  {}", info.num_shader_cores);
            println!("  Memory:        {} KB", info.num_l2_bytes / 1024);

            match info.vendor {
                gpu_info::GpuVendor::Mali => {
                    println!("  Type:          ARM Mali");
                    if let Some(mali) = info.mali_data {
                        println!("  GPU ID:        0x{:04X}", mali.gpu_id);
                    }
                }
                gpu_info::GpuVendor::Adreno => {
                    println!("  Type:          Qualcomm Adreno");
                    if let Some(adreno) = info.adreno_data {
                        println!("  Chip ID:       0x{:08X}", adreno.chip_id);
                    }
                }
                _ => {
                    println!("  Type:          Unknown");
                }
            }
        }
        Err(e) => {
            println!("❌ No GPU detected: {}", e);
            println!("   Try specifying a device path manually");
        }
    }

    Ok(())
}