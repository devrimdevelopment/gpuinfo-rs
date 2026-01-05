//! Compares Parity vs Extended mode performance
use armgpuinfo::{query_mali_with_mode, Mode};
use std::path::Path;
use std::time::{Instant, Duration};

fn benchmark_mali_modes() {
    println!("📊 Mali Mode Comparison");
    println!("======================\n");
    
    let test_path = "/dev/mali0";
    
    if !Path::new(test_path).exists() {
        println!("❌ Mali device not found, skipping...");
        return;
    }
    
    // Warmup
    let _ = query_mali_with_mode(test_path, Mode::Parity);
    
    const ITERATIONS: usize = 100;
    let mut parity_times = Vec::with_capacity(ITERATIONS);
    let mut extended_times = Vec::with_capacity(ITERATIONS);
    
    // Benchmark Parity mode
    println!("Benchmarking Parity mode...");
    for i in 0..ITERATIONS {
        let start = Instant::now();
        let _ = query_mali_with_mode(test_path, Mode::Parity);
        parity_times.push(start.elapsed());
        
        if i % 20 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
    println!();
    
    // Benchmark Extended mode  
    println!("Benchmarking Extended mode...");
    for i in 0..ITERATIONS {
        let start = Instant::now();
        let _ = query_mali_with_mode(test_path, Mode::Extended);
        extended_times.push(start.elapsed());
        
        if i % 20 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
    println!();
    
    let avg_parity = average_duration(&parity_times);
    let avg_extended = average_duration(&extended_times);
    
    println!("\nResults ({} iterations):", ITERATIONS);
    println!("  Parity mode:   {:?} avg", avg_parity);
    println!("  Extended mode: {:?} avg", avg_extended);
    
    if avg_parity > Duration::from_nanos(0) {
        let diff = avg_extended.as_secs_f64() / avg_parity.as_secs_f64();
        println!("  Extended is {:.1}x {}", diff, if diff > 1.0 { "slower" } else { "faster" });
    }
    
    // Memory usage comparison
    println!("\n💾 Memory characteristics:");
    match query_mali_with_mode(test_path, Mode::Parity) {
        Ok(info) => {
            let size = std::mem::size_of_val(&info);
            println!("  GpuInfo struct size: {} bytes", size);
            println!("  gpu_name: {} ({})", 
                info.gpu_name,
                if matches!(info.gpu_name, std::borrow::Cow::Borrowed(_)) {
                    "borrowed, no heap allocation"
                } else {
                    "owned on heap"
                });
        }
        Err(e) => println!("  Error: {}", e),
    }
}

fn average_duration(durations: &[Duration]) -> Duration {
    let sum: Duration = durations.iter().sum();
    sum / durations.len() as u32
}

fn main() {
    println!("🚀 Strategy Pattern Performance Comparison");
    println!("=========================================\n");
    
    benchmark_mali_modes();
}