//! Demonstration of Adreno GPU query functionality

use gpu_info::{query_adreno, GpuResult};

fn main() -> GpuResult<()> {
    println!("Adreno GPU Info Demo");
    println!("====================\n");

    match query_adreno("/dev/kgsl-3d0") {
        Ok(info) => {
            println!("✅ Adreno GPU detected:");
            println!("  Vendor:        {}", info.vendor);
            println!("  Name:          {}", info.gpu_name);
            println!("  Architecture:  {} ({}.{})",
                info.architecture, info.architecture_major, info.architecture_minor);
            println!("  Shader Cores:  {}", info.num_shader_cores);
            println!("  GMEM Size:     {} KB", info.num_l2_bytes / 1024);
            println!("  Bus Width:     {} bits", info.num_bus_bits);

            if let Some(adreno) = info.adreno_data {
                println!("  Chip ID:       0x{:08X}", adreno.chip_id);
                println!("  Confidence:    {}", adreno.spec_confidence);
                println!("  Stream Procs:  {}", adreno.stream_processors);
                println!("  Max Freq:      {} MHz", adreno.max_freq_mhz);
            }
        }
        Err(e) => {
            println!("❌ Failed to query Adreno GPU: {}", e);
            println!("   Make sure /dev/kgsl-3d0 exists and is accessible");
        }
    }

    Ok(())
}