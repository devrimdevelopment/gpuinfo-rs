//! Benchmark comparison: String vs Cow performance
use armgpuinfo::{GpuInfoBuilder, GpuInfo, GpuVendor};
use std::borrow::Cow;
use std::time::{Instant, Duration};

fn main() {
    const ITERATIONS: usize = 100_000;
    
    println!("🚀 Benchmark: String vs Cow<'static, str>");
    println!("Iterations: {}\n", ITERATIONS);
    
    // Test 1: With String (heap allocations)
    println!("Testing String version...");
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let _ = GpuInfoBuilder::default()
            .gpu_name(format!("Mali-G{}", i % 100))
            .architecture("Valhall".to_string())
            .gpu_id(0x9000)
            .raw_gpu_id(0x9000000000000000)
            .num_shader_cores(16)
            .num_l2_bytes(2_097_152)
            .build();
    }
    let string_duration = start.elapsed();
    
    // Test 2: With Cow (no allocations for static strings)
    println!("Testing Cow version...");
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let _ = GpuInfoBuilder::default()
            .gpu_name(Cow::Borrowed("Mali-G710"))
            .architecture(Cow::Borrowed("Valhall"))
            .gpu_id(0x9000)
            .raw_gpu_id(0x9000000000000000)
            .num_shader_cores(16)
            .num_l2_bytes(2_097_152)
            .build();
    }
    let cow_duration = start.elapsed();
    
    println!("\nResults:");
    println!("  String version: {:?}", string_duration);
    println!("  Cow version:    {:?}", cow_duration);
    
    if cow_duration > Duration::from_nanos(0) {
        let speedup = string_duration.as_secs_f64() / cow_duration.as_secs_f64();
        println!("  Speedup:        {:.1}x faster", speedup);
    }
}