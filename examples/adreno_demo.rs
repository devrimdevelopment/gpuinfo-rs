//! Demonstrates Adreno GPU queries
use armgpuinfo::query_adreno;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let device_path = "/dev/kgsl-3d0";
    
    if !Path::new(device_path).exists() {
        println!("❌ Adreno device not found at {}", device_path);
        println!("   Try on a system with Qualcomm GPU");
        return Ok(());
    }
    
    println!("🎮 Adreno GPU Query Demonstration");
    println!("================================\n");
    
    println!("Querying Adreno GPU...");
    match query_adreno(device_path) {
        Ok(info) => {
            println!("✅ Success!");
            println!("\n📊 GPU Information:");
            println!("   Name: {}", info.gpu_name);
            println!("   Vendor: {}", info.vendor);
            println!("   Architecture: {}", info.architecture);
            println!("   Arch Version: {}.{}", info.architecture_major, info.architecture_minor);
            println!("   Shader Cores: {}", info.num_shader_cores);
            println!("   L2 Cache: {} KB", info.num_l2_bytes / 1024);
            println!("   Bus Width: {} bits", info.num_bus_bits);
            
            // WICHTIG: "ref" oder "&" verwenden, um zu borrowen
            if let Some(ref adreno) = info.adreno_data {  // 👈 "ref" hinzufügen
                println!("\n🔧 Adreno Details:");
                println!("   Chip ID: 0x{:08X}", adreno.chip_id);
                println!("   GPU Model Code: 0x{:08X}", adreno.gpu_model_code);
                println!("   MMU Enabled: {}", adreno.mmu_enabled);
                println!("   GMEM Size: {} bytes", adreno.gmem_size_bytes);
                println!("   Confidence: {}", adreno.spec_confidence);
                println!("   Stream Processors: {}", adreno.stream_processors);
                println!("   Max Frequency: {} MHz", adreno.max_freq_mhz);
                println!("   Process: {} nm", adreno.process_nm);
                println!("   Release Year: {}", adreno.release_year);
                
                if !adreno.snapdragon_models.is_empty() {
                    println!("   Snapdragon Models: {}", adreno.snapdragon_models.join(", "));
                }
            }
            
            println!("\n⚡ Performance:");
            println!("   FP16 Support: {}", info.supports_fp16());
            
            // Beispiel FLOPS-Berechnung bei 800 MHz
            let freq_mhz = 800;
            let flops = info.calculate_fp32_flops(freq_mhz * 1_000_000);
            println!("   FP32 FLOPS @ {} MHz: {:.1} GFLOPS", 
                freq_mhz, flops as f64 / 1_000_000_000.0);
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
    
    Ok(())
}