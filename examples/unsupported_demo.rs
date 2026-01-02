// examples/unsupported_demo.rs
use armgpuinfo::{GpuError, query_gpu_auto};

fn main() {
    println!("ARM GPU Info - Unsupported Device Demo");
    println!("======================================\n");
    
    match query_gpu_auto(None::<&str>) {
        Ok(info) => {
            println!("✅ Supported GPU Found:");
            println!("   Name: {}", info.gpu_name);
            println!("   Vendor: {:?}", info.vendor);
            println!("   Cores: {}", info.num_shader_cores);
        }
        
        Err(GpuError::UnsupportedGpu { id, cores }) => {
            println!("❌ UNSUPPORTED GPU DETECTED");
            println!("\nDetails:");
            println!("   Chip ID: 0x{:08X}", id);
            println!("   Reported cores: {}", cores);
            
            let arch_major = (id >> 24) & 0xFF;
            let arch_minor = (id >> 16) & 0xFF;
            println!("   Architecture: Adreno {}{}x", arch_major, arch_minor);
            
            println!("\nNext steps:");
            println!("   1. Report this at: https://github.com/.../issues");
            println!("   2. Include the chip ID above");
            println!("   3. Help expand the database!");
        }
        
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
}