//! Demonstration of Parity vs Extended mode differences for Mali GPUs

use gpu_info::{query_gpu_with_mode, GpuResult, Mode};
use std::env;

fn main() -> GpuResult<()> {
    println!("ARM Mali GPU Info - Mode Comparison Demo");
    println!("=========================================\n");

    let device_path = env::args().nth(1).unwrap_or_else(|| "/dev/mali0".to_string());

    println!("Testing device: {}", device_path);
    println!();

    // Try Parity mode first
    match query_gpu_with_mode(&device_path, Mode::Parity) {
        Ok(info) => {
            println!("✅ PARITY MODE (libgpuinfo clone):");
            println!("  Name:           {}", info.gpu_name);
            println!("  Architecture:   {}", info.architecture);
            println!("  GPU ID:         0x{:04X}",
                info.mali_data.as_ref().map(|m| m.gpu_id).unwrap_or(0));
            println!("  Shader Cores:   {}", info.num_shader_cores);
            println!("  L2 Cache:       {} KB", info.num_l2_bytes / 1024);
            println!("  Bus Width:      {}",
                if info.num_bus_bits > 0 { format!("{} bits", info.num_bus_bits) }
                else { "Not available".to_string() });
        }
        Err(e) => {
            println!("❌ PARITY MODE failed: {}", e);
        }
    }

    println!("---");

    // Try Extended mode
    match query_gpu_with_mode(&device_path, Mode::Extended) {
        Ok(info) => {
            println!("✅ EXTENDED MODE (enhanced features):");
            println!("  Name:           {}", info.gpu_name);
            println!("  Architecture:   {} ({}.{})",
                info.architecture, info.architecture_major, info.architecture_minor);
            println!("  GPU ID:         0x{:04X}",
                info.mali_data.as_ref().map(|m| m.gpu_id).unwrap_or(0));
            println!("  Shader Cores:   {}", info.num_shader_cores);
            println!("  L2 Cache:       {} KB", info.num_l2_bytes / 1024);

            if let Some(mali) = info.mali_data {
                println!("  Execution Eng.: {} per core", mali.num_exec_engines);
                println!("  FP32 FMAs/Core: {}", mali.num_fp32_fmas_per_core);
                println!("  FP16 FMAs/Core: {}", mali.num_fp16_fmas_per_core);
                println!("  Texels/Core:    {}", mali.num_texels_per_core);
                println!("  Pixels/Core:    {}", mali.num_pixels_per_core);
            }

            println!("  Bus Width:      {} bits", info.num_bus_bits);
            // println!("  FP16 Support:   {}", if info.supports_fp16() { "✅ Yes" } else { "❌ No" }); // to fix
        }
        Err(e) => {
            println!("❌ EXTENDED MODE failed: {}", e);
        }
    }

    Ok(())
}