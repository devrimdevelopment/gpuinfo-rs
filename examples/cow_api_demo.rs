//! Demonstrates the flexible Cow-based API
use armgpuinfo::{GpuInfo, GpuVendor, GpuInfoBuilder};
use std::borrow::Cow;

fn main() {
    println!("🔄 Cow<'static, str> API Demonstration");
    println!("=====================================\n");
    
    // Example 1: Builder with different string types
    println!("1. Builder accepts different string types:");
    
    // &'static str
    let builder1 = GpuInfoBuilder::default()
        .gpu_name("Mali-G710")
        .architecture("Valhall");
    
    // String
    let builder2 = GpuInfoBuilder::default()
        .gpu_name(String::from("Adreno 740"))
        .architecture("Adreno 7xx".to_string());
    
    // Cow explicitly
    let builder3 = GpuInfoBuilder::default()
        .gpu_name(Cow::Borrowed("Mali-G57"))
        .architecture(Cow::Owned(String::from("Valhall")));
    
    println!("   All builders created successfully!");
    
    // Example 2: Manual GpuInfo creation
    println!("\n2. Manual GpuInfo creation:");
    
    let gpu1 = GpuInfo {
        vendor: GpuVendor::Mali,
        gpu_name: "Test-GPU".into(),
        architecture: "Test-Arch".into(),
        architecture_major: 1,
        architecture_minor: 0,
        num_shader_cores: 4,
        num_l2_bytes: 1024,
        num_bus_bits: 64,
        mali_data: None,
        adreno_data: None,
    };
    
    println!("   Created: {}", gpu1);
    
    // Example 3: Check Cow variant
    println!("\n3. Checking Cow variant at runtime:");
    
    let gpus = vec![
        ("Static", "Mali-G710".into()),
        ("Dynamic", String::from("Custom-GPU").into()),
    ];
    
    for (name, cow) in gpus {
        match &cow {
            Cow::Borrowed(s) => println!("   {}: Borrowed '{}'", name, s),
            Cow::Owned(s) => println!("   {}: Owned '{}'", name, s),
        }
    }
}