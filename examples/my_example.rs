//! Simple demonstration of all query methods

use armgpuinfo::{query_gpu_auto, GpuError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match query_gpu_auto(None::<&str>) {
        Ok(info) => {
            println!("   Name: {}", info.gpu_name);
            println!("   Vendor: {}", info.vendor);
            
            // Show additional info based on vendor
            match info.vendor {
                armgpuinfo::GpuVendor::Mali => {
                    if let Some(mali) = &info.mali_data {
                        println!("   GPU ID: 0x{:08X}", mali.gpu_id);
                    }
                    println!("   💡 Run: cargo run --example mode_demo");
                }
                armgpuinfo::GpuVendor::Adreno => {
                    if let Some(adreno) = &info.adreno_data {
                        println!("   Chip ID: 0x{:08X}", adreno.chip_id);
                        println!("   Confidence: {}", adreno.spec_confidence);
                    }
                    println!("   💡 Run: cargo run --example adreno_demo");
                }
                _ => {}
            }
        }
        Err(GpuError::DeviceNotFound) => {
            println!("❌ No GPU device found");
        }
        Err(GpuError::PermissionDenied) => {
            println!("❌ Permission denied");
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
    
    
    Ok(())
}