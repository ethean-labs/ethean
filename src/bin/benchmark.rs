//! Beam Chain Performance Benchmark Binary
//!
//! Runs comprehensive performance benchmarks for the Beam Chain consensus implementation.

use panro::bench::BeamChainBenchmark;
use std::env;

fn main() {
    println!("🔥 Beam Chain Performance Benchmark Suite 🔥\n");
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("full");
    
    let mut benchmark = BeamChainBenchmark::new();
    
    match mode {
        "bls" => {
            println!("Running BLS operations benchmark...");
            let result = benchmark.benchmark_bls_operations(1000);
            println!("\n📊 BLS Benchmark Results:\n{}", result);
        },
        "attestation" => {
            println!("Running attestation processing benchmark...");
            let result = benchmark.benchmark_attestation_processing(500);
            println!("\n📊 Attestation Benchmark Results:\n{}", result);
        },
        "validator" => {
            println!("Running validator management benchmark...");
            let result = benchmark.benchmark_validator_management(300);
            println!("\n📊 Validator Benchmark Results:\n{}", result);
        },
        "memory" => {
            println!("Running memory usage analysis...");
            let result = benchmark.analyze_memory_usage();
            println!("\n📊 Memory Analysis Results:\n{}", result);
        },
        "full" | _ => {
            println!("Running complete benchmark suite...");
            let result = benchmark.run_full_benchmark();
            println!("\n{}", result);
        }
    }
    
    println!("\n🎯 Benchmark completed successfully!");
    
    // Generate BLS performance report
    println!("\n📈 BLS Performance Details:");
    println!("{}", benchmark.bls_aggregator.generate_performance_report());
    
    // Show cache statistics
    let cache_ratio = benchmark.bls_aggregator.get_cache_hit_ratio();
    if cache_ratio > 0.0 {
        println!("\n💾 Cache Performance:");
        println!("- Cache hit ratio: {:.2}%", cache_ratio * 100.0);
        println!("- Operations per second: {:.2}", benchmark.bls_aggregator.get_operations_per_second());
    }
    
    println!("\n✨ Use the following commands for specific benchmarks:");
    println!("cargo run --bin benchmark bls        # BLS operations only");
    println!("cargo run --bin benchmark attestation # Attestation processing only");
    println!("cargo run --bin benchmark validator   # Validator management only");
    println!("cargo run --bin benchmark memory      # Memory analysis only");
    println!("cargo run --bin benchmark full       # Complete suite (default)");
}
