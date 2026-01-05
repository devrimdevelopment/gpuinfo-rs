//! Demonstrates Mali query modes
use armgpuinfo::{query_mali_with_mode, Mode};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let device_path = "/dev/mali0";
    
    if !Path::new(device_path).exists() {
        println!("❌ Mali device not found at {}", device_path);
        println!("   Try on a system with Mali GPU or use a different path");
        return Ok(());
    }
    
    println!("🎛️ Mali Query Mode Demonstration");
    println!("===============================\n");
    
    // Parity Mode (default)
    println!("1. Parity Mode (fast, lenient):");
    match query_mali_with_mode(device_path, Mode::Parity) {
        Ok(info) => {
            println!("   ✅ Success!");
            println!("   Name: {}", info.gpu_name);
            println!("   Architecture: {}", info.architecture);
            println!("   Cores: {}", info.num_shader_cores);
            
            // WICHTIG: &info.mali_data (Borrow) statt info.mali_data (Move)
            if let Some(ref mali) = info.mali_data {  // 👈 "ref" hinzufügen
                println!("   GPU ID: 0x{:08X}", mali.gpu_id);
                println!("   L2 Slices: {}", mali.num_l2_slices);
            }
            
            // Jetzt kannst du info weiter verwenden
            println!("   L2 Cache: {} KB", info.num_l2_bytes / 1024);
        }
        Err(e) => println!("   ❌ Error: {}", e),
    }
    
    println!("\n2. Extended Mode (strict, full features):");
    match query_mali_with_mode(device_path, Mode::Extended) {
        Ok(info) => {
            println!("   ✅ Success!");
            println!("   Name: {}", info.gpu_name);
            println!("   Architecture: {}", info.architecture);
            println!("   Cores: {}", info.num_shader_cores);
            println!("   L2 Cache: {} KB", info.num_l2_bytes / 1024);
            println!("   Bus Width: {} bits", info.num_bus_bits);
            
            // WICHTIG: &info.mali_data (Borrow) statt info.mali_data (Move)
            if let Some(ref mali) = info.mali_data {  // 👈 "ref" hinzufügen
                println!("   GPU ID: 0x{:08X}", mali.gpu_id);
                println!("   Exec Engines: {}", mali.num_exec_engines);
                println!("   FP32 FMAs/Core: {}", mali.num_fp32_fmas_per_core);
                println!("   FP16 FMAs/Core: {}", mali.num_fp16_fmas_per_core);
            }
            
            // Jetzt kannst du info weiter verwenden
            println!("   FP16 Support: {}", info.supports_fp16());
        }
        Err(e) => println!("   ❌ Error: {}", e),
    }
    
    println!("\n💡 Summary:");
    println!("• Parity: Faster, more tolerant, basic info");
    println!("• Extended: More validation, detailed specs");
    
    Ok(())
}