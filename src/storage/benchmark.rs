//! Database performance benchmarking module
//!
//! Provides comprehensive performance testing and monitoring tools
//! for database operations, including throughput, latency, and optimization metrics.

use std::time::{Duration, Instant};
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use rand::{Rng, thread_rng};
use crate::storage::database::{Database, DatabaseError};
use crate::storage::cache::LruCache;

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Number of operations per test
    pub operations_count: usize,
    /// Number of concurrent threads
    pub concurrency: usize,
    /// Key size in bytes
    pub key_size: usize,
    /// Value size in bytes
    pub value_size: usize,
    /// Batch size for batch operations
    pub batch_size: usize,
    /// Warmup operations before measurement
    pub warmup_operations: usize,
    /// Read/write ratio (0.0 = all writes, 1.0 = all reads)
    pub read_write_ratio: f64,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            operations_count: 10000,
            concurrency: 8,
            key_size: 32,
            value_size: 256,
            batch_size: 100,
            warmup_operations: 1000,
            read_write_ratio: 0.7, // 70% reads, 30% writes
        }
    }
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub operation_type: String,
    pub total_operations: usize,
    pub duration_ms: u64,
    pub throughput_ops_per_sec: f64,
    pub avg_latency_us: f64,
    pub min_latency_us: u64,
    pub max_latency_us: u64,
    pub p50_latency_us: u64,
    pub p95_latency_us: u64,
    pub p99_latency_us: u64,
    pub error_count: usize,
    pub data_volume_mb: f64,
}

/// Benchmark suite
pub struct BenchmarkSuite {
    database: Arc<Database>,
    cache: Option<Arc<RwLock<LruCache<Vec<u8>, Vec<u8>>>>>,
    config: BenchmarkConfig,
    metrics: Vec<PerformanceMetrics>,
}

impl BenchmarkSuite {
    /// Create new benchmark suite
    pub fn new(database: Arc<Database>, config: BenchmarkConfig) -> Self {
        Self {
            database,
            cache: None,
            config,
            metrics: Vec::new(),
        }
    }
    
    /// Add cache layer for testing
    pub fn with_cache(mut self, cache: Arc<RwLock<LruCache<Vec<u8>, Vec<u8>>>>) -> Self {
        self.cache = Some(cache);
        self
    }
    
    /// Run complete benchmark suite
    pub async fn run_full_suite(&mut self) -> Result<BenchmarkReport, DatabaseError> {
        println!("🚀 Starting Database Performance Benchmark Suite");
        println!("Configuration: {:?}", self.config);
        
        // Warmup phase
        self.warmup().await?;
        
        // Run individual benchmarks
        self.benchmark_sequential_writes().await?;
        self.benchmark_sequential_reads().await?;
        self.benchmark_random_writes().await?;
        self.benchmark_random_reads().await?;
        self.benchmark_batch_operations().await?;
        self.benchmark_concurrent_operations().await?;
        self.benchmark_mixed_workload().await?;
        
        if self.cache.is_some() {
            self.benchmark_cache_performance().await?;
        }
        
        // Generate report
        let report = BenchmarkReport {
            config: self.config.clone(),
            metrics: self.metrics.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        println!("✅ Benchmark suite completed");
        Ok(report)
    }
    
    /// Warmup database
    async fn warmup(&self) -> Result<(), DatabaseError> {
        println!("🔥 Warming up database...");
        
        let mut rng = thread_rng();
        for i in 0..self.config.warmup_operations {
            let key = self.generate_key(i);
            let value = self.generate_value(&mut rng);
            
            self.database.put(&key, &value)?;
            
            if i % 100 == 0 {
                let _ = self.database.get(&key)?;
            }
        }
        
        println!("✅ Warmup completed: {} operations", self.config.warmup_operations);
        Ok(())
    }
    
    /// Benchmark sequential writes
    async fn benchmark_sequential_writes(&mut self) -> Result<(), DatabaseError> {
        println!("📝 Running sequential writes benchmark...");
        
        let mut latencies = Vec::new();
        let mut rng = thread_rng();
        let start_time = Instant::now();
        let mut error_count = 0;
        
        for i in 0..self.config.operations_count {
            let key = self.generate_key(i);
            let value = self.generate_value(&mut rng);
            
            let op_start = Instant::now();
            match self.database.put(&key, &value) {
                Ok(_) => {
                    let latency = op_start.elapsed().as_micros() as u64;
                    latencies.push(latency);
                }
                Err(_) => error_count += 1,
            }
        }
        
        let duration = start_time.elapsed();
        let metrics = self.calculate_metrics(
            "sequential_writes",
            latencies,
            duration,
            error_count,
        );
        
        self.metrics.push(metrics);
        println!("✅ Sequential writes: {:.2} ops/sec", self.metrics.last().unwrap().throughput_ops_per_sec);
        
        Ok(())
    }
    
    /// Benchmark sequential reads
    async fn benchmark_sequential_reads(&mut self) -> Result<(), DatabaseError> {
        println!("📖 Running sequential reads benchmark...");
        
        let mut latencies = Vec::new();
        let start_time = Instant::now();
        let mut error_count = 0;
        let mut found_count = 0;
        
        for i in 0..self.config.operations_count {
            let key = self.generate_key(i);
            
            let op_start = Instant::now();
            match self.database.get(&key) {
                Ok(Some(_)) => {
                    let latency = op_start.elapsed().as_micros() as u64;
                    latencies.push(latency);
                    found_count += 1;
                }
                Ok(None) => {
                    // Key not found, still count as successful operation
                    let latency = op_start.elapsed().as_micros() as u64;
                    latencies.push(latency);
                }
                Err(_) => error_count += 1,
            }
        }
        
        let duration = start_time.elapsed();
        let metrics = self.calculate_metrics(
            "sequential_reads",
            latencies,
            duration,
            error_count,
        );
        
        self.metrics.push(metrics);
        println!("✅ Sequential reads: {:.2} ops/sec, {}/{} found", 
                self.metrics.last().unwrap().throughput_ops_per_sec, 
                found_count, 
                self.config.operations_count);
        
        Ok(())
    }
    
    /// Benchmark random writes
    async fn benchmark_random_writes(&mut self) -> Result<(), DatabaseError> {
        println!("🎲 Running random writes benchmark...");
        
        let mut latencies = Vec::new();
        let mut rng = thread_rng();
        let start_time = Instant::now();
        let mut error_count = 0;
        
        for _ in 0..self.config.operations_count {
            let key_id = rng.gen_range(0..self.config.operations_count * 2);
            let key = self.generate_key(key_id);
            let value = self.generate_value(&mut rng);
            
            let op_start = Instant::now();
            match self.database.put(&key, &value) {
                Ok(_) => {
                    let latency = op_start.elapsed().as_micros() as u64;
                    latencies.push(latency);
                }
                Err(_) => error_count += 1,
            }
        }
        
        let duration = start_time.elapsed();
        let metrics = self.calculate_metrics(
            "random_writes",
            latencies,
            duration,
            error_count,
        );
        
        self.metrics.push(metrics);
        println!("✅ Random writes: {:.2} ops/sec", self.metrics.last().unwrap().throughput_ops_per_sec);
        
        Ok(())
    }
    
    /// Benchmark random reads
    async fn benchmark_random_reads(&mut self) -> Result<(), DatabaseError> {
        println!("🔍 Running random reads benchmark...");
        
        let mut latencies = Vec::new();
        let mut rng = thread_rng();
        let start_time = Instant::now();
        let mut error_count = 0;
        let mut found_count = 0;
        
        for _ in 0..self.config.operations_count {
            let key_id = rng.gen_range(0..self.config.operations_count * 2);
            let key = self.generate_key(key_id);
            
            let op_start = Instant::now();
            match self.database.get(&key) {
                Ok(Some(_)) => {
                    let latency = op_start.elapsed().as_micros() as u64;
                    latencies.push(latency);
                    found_count += 1;
                }
                Ok(None) => {
                    let latency = op_start.elapsed().as_micros() as u64;
                    latencies.push(latency);
                }
                Err(_) => error_count += 1,
            }
        }
        
        let duration = start_time.elapsed();
        let metrics = self.calculate_metrics(
            "random_reads",
            latencies,
            duration,
            error_count,
        );
        
        self.metrics.push(metrics);
        println!("✅ Random reads: {:.2} ops/sec, {}/{} found", 
                self.metrics.last().unwrap().throughput_ops_per_sec, 
                found_count, 
                self.config.operations_count);
        
        Ok(())
    }
    
    /// Benchmark batch operations
    async fn benchmark_batch_operations(&mut self) -> Result<(), DatabaseError> {
        println!("📦 Running batch operations benchmark...");
        
        let mut latencies = Vec::new();
        let mut rng = thread_rng();
        let start_time = Instant::now();
        let mut error_count = 0;
        let mut total_ops = 0;
        
        let batch_count = self.config.operations_count / self.config.batch_size;
        
        for batch_id in 0..batch_count {
            let mut batch = Vec::new();
            
            for i in 0..self.config.batch_size {
                let key_id = batch_id * self.config.batch_size + i;
                let key = self.generate_key(key_id);
                let value = self.generate_value(&mut rng);
                batch.push((key, value));
            }
            
            let op_start = Instant::now();
            match self.database.batch_put(&batch) {
                Ok(_) => {
                    let latency = op_start.elapsed().as_micros() as u64;
                    latencies.push(latency);
                    total_ops += batch.len();
                }
                Err(_) => error_count += 1,
            }
        }
        
        let duration = start_time.elapsed();
        let metrics = PerformanceMetrics {
            operation_type: "batch_operations".to_string(),
            total_operations: total_ops,
            duration_ms: duration.as_millis() as u64,
            throughput_ops_per_sec: total_ops as f64 / duration.as_secs_f64(),
            avg_latency_us: latencies.iter().sum::<u64>() as f64 / latencies.len() as f64,
            min_latency_us: *latencies.iter().min().unwrap_or(&0),
            max_latency_us: *latencies.iter().max().unwrap_or(&0),
            p50_latency_us: self.percentile(&latencies, 50),
            p95_latency_us: self.percentile(&latencies, 95),
            p99_latency_us: self.percentile(&latencies, 99),
            error_count,
            data_volume_mb: (total_ops * (self.config.key_size + self.config.value_size)) as f64 / 1024.0 / 1024.0,
        };
        
        self.metrics.push(metrics);
        println!("✅ Batch operations: {:.2} ops/sec", self.metrics.last().unwrap().throughput_ops_per_sec);
        
        Ok(())
    }
    
    /// Benchmark concurrent operations
    async fn benchmark_concurrent_operations(&mut self) -> Result<(), DatabaseError> {
        println!("🔄 Running concurrent operations benchmark...");
        
        let start_time = Instant::now();
        let mut handles = Vec::new();
        let ops_per_thread = self.config.operations_count / self.config.concurrency;
        
        for thread_id in 0..self.config.concurrency {
            let database = self.database.clone();
            let config = self.config.clone();
            
            let handle = tokio::spawn(async move {
                let mut latencies = Vec::new();
                let mut rng = thread_rng();
                let mut error_count = 0;
                
                for i in 0..ops_per_thread {
                    let key_id = thread_id * ops_per_thread + i;
                    let key = format!("concurrent_key_{}", key_id).into_bytes();
                    
                    let op_start = Instant::now();
                    
                    if rng.gen::<f64>() < config.read_write_ratio {
                        // Read operation
                        match database.get(&key) {
                            Ok(_) => {
                                let latency = op_start.elapsed().as_micros() as u64;
                                latencies.push(latency);
                            }
                            Err(_) => error_count += 1,
                        }
                    } else {
                        // Write operation
                        let value = (0..config.value_size).map(|_| rng.gen::<u8>()).collect::<Vec<u8>>();
                        match database.put(&key, &value) {
                            Ok(_) => {
                                let latency = op_start.elapsed().as_micros() as u64;
                                latencies.push(latency);
                            }
                            Err(_) => error_count += 1,
                        }
                    }
                }
                
                (latencies, error_count)
            });
            
            handles.push(handle);
        }
        
        // Collect results
        let mut all_latencies = Vec::new();
        let mut total_errors = 0;
        
        for handle in handles {
            let (latencies, errors) = handle.await.unwrap();
            all_latencies.extend(latencies);
            total_errors += errors;
        }
        
        let duration = start_time.elapsed();
        let metrics = self.calculate_metrics(
            "concurrent_operations",
            all_latencies,
            duration,
            total_errors,
        );
        
        self.metrics.push(metrics);
        println!("✅ Concurrent operations: {:.2} ops/sec", self.metrics.last().unwrap().throughput_ops_per_sec);
        
        Ok(())
    }
    
    /// Benchmark mixed workload
    async fn benchmark_mixed_workload(&mut self) -> Result<(), DatabaseError> {
        println!("🎯 Running mixed workload benchmark...");
        
        let mut latencies = Vec::new();
        let mut rng = thread_rng();
        let start_time = Instant::now();
        let mut error_count = 0;
        let mut read_count = 0;
        let mut write_count = 0;
        
        for i in 0..self.config.operations_count {
            let op_start = Instant::now();
            
            if rng.gen::<f64>() < self.config.read_write_ratio {
                // Read operation
                let key_id = rng.gen_range(0..i.max(1));
                let key = self.generate_key(key_id);
                
                match self.database.get(&key) {
                    Ok(_) => {
                        let latency = op_start.elapsed().as_micros() as u64;
                        latencies.push(latency);
                        read_count += 1;
                    }
                    Err(_) => error_count += 1,
                }
            } else {
                // Write operation
                let key = self.generate_key(i);
                let value = self.generate_value(&mut rng);
                
                match self.database.put(&key, &value) {
                    Ok(_) => {
                        let latency = op_start.elapsed().as_micros() as u64;
                        latencies.push(latency);
                        write_count += 1;
                    }
                    Err(_) => error_count += 1,
                }
            }
        }
        
        let duration = start_time.elapsed();
        let metrics = self.calculate_metrics(
            "mixed_workload",
            latencies,
            duration,
            error_count,
        );
        
        self.metrics.push(metrics);
        println!("✅ Mixed workload: {:.2} ops/sec (reads: {}, writes: {})", 
                self.metrics.last().unwrap().throughput_ops_per_sec, 
                read_count, 
                write_count);
        
        Ok(())
    }
    
    /// Benchmark cache performance
    async fn benchmark_cache_performance(&mut self) -> Result<(), DatabaseError> {
        if let Some(cache) = &self.cache {
            println!("💾 Running cache performance benchmark...");
            
            let mut latencies = Vec::new();
            let mut rng = thread_rng();
            let start_time = Instant::now();
            let mut cache_hits = 0;
            let mut cache_misses = 0;
            
            // Populate cache
            for i in 0..self.config.operations_count / 2 {
                let key = self.generate_key(i);
                let value = self.generate_value(&mut rng);
                cache.write().await.put(key, value);
            }
            
            // Test cache performance
            for i in 0..self.config.operations_count {
                let key = self.generate_key(i);
                let op_start = Instant::now();
                
                if cache.read().await.get(&key).is_some() {
                    cache_hits += 1;
                } else {
                    cache_misses += 1;
                }
                
                let latency = op_start.elapsed().as_micros() as u64;
                latencies.push(latency);
            }
            
            let duration = start_time.elapsed();
            let mut metrics = self.calculate_metrics(
                "cache_performance",
                latencies,
                duration,
                0,
            );
            
            // Add cache-specific metrics
            metrics.operation_type = format!(
                "cache_performance (hit_rate: {:.2}%)",
                cache_hits as f64 / (cache_hits + cache_misses) as f64 * 100.0
            );
            
            self.metrics.push(metrics);
            println!("✅ Cache performance: {:.2} ops/sec, hit rate: {:.2}%", 
                    self.metrics.last().unwrap().throughput_ops_per_sec,
                    cache_hits as f64 / (cache_hits + cache_misses) as f64 * 100.0);
        }
        
        Ok(())
    }
    
    // Helper methods
    
    fn generate_key(&self, id: usize) -> Vec<u8> {
        let key_str = format!("bench_key_{:0width$}", id, width = self.config.key_size - 10);
        key_str.into_bytes()
    }
    
    fn generate_value(&self, rng: &mut impl Rng) -> Vec<u8> {
        (0..self.config.value_size).map(|_| rng.gen::<u8>()).collect()
    }
    
    fn calculate_metrics(
        &self,
        operation_type: &str,
        mut latencies: Vec<u64>,
        duration: Duration,
        error_count: usize,
    ) -> PerformanceMetrics {
        latencies.sort_unstable();
        
        let total_operations = latencies.len();
        let throughput = total_operations as f64 / duration.as_secs_f64();
        let avg_latency = latencies.iter().sum::<u64>() as f64 / latencies.len() as f64;
        
        PerformanceMetrics {
            operation_type: operation_type.to_string(),
            total_operations,
            duration_ms: duration.as_millis() as u64,
            throughput_ops_per_sec: throughput,
            avg_latency_us: avg_latency,
            min_latency_us: latencies.first().copied().unwrap_or(0),
            max_latency_us: latencies.last().copied().unwrap_or(0),
            p50_latency_us: self.percentile(&latencies, 50),
            p95_latency_us: self.percentile(&latencies, 95),
            p99_latency_us: self.percentile(&latencies, 99),
            error_count,
            data_volume_mb: (total_operations * (self.config.key_size + self.config.value_size)) as f64 / 1024.0 / 1024.0,
        }
    }
    
    fn percentile(&self, sorted_latencies: &[u64], percentile: u8) -> u64 {
        if sorted_latencies.is_empty() {
            return 0;
        }
        
        let index = (sorted_latencies.len() as f64 * percentile as f64 / 100.0) as usize;
        let index = index.min(sorted_latencies.len() - 1);
        sorted_latencies[index]
    }
}

/// Benchmark report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub config: BenchmarkConfig,
    pub metrics: Vec<PerformanceMetrics>,
    pub timestamp: u64,
}

impl BenchmarkReport {
    /// Print human-readable report
    pub fn print_report(&self) {
        println!("\n📊 DATABASE PERFORMANCE BENCHMARK REPORT");
        println!("==========================================");
        println!("Timestamp: {}", self.timestamp);
        println!("Configuration: {:?}\n", self.config);
        
        for metric in &self.metrics {
            println!("🔹 {}", metric.operation_type);
            println!("  Operations: {}", metric.total_operations);
            println!("  Duration: {}ms", metric.duration_ms);
            println!("  Throughput: {:.2} ops/sec", metric.throughput_ops_per_sec);
            println!("  Avg Latency: {:.2}μs", metric.avg_latency_us);
            println!("  P50 Latency: {}μs", metric.p50_latency_us);
            println!("  P95 Latency: {}μs", metric.p95_latency_us);
            println!("  P99 Latency: {}μs", metric.p99_latency_us);
            println!("  Data Volume: {:.2}MB", metric.data_volume_mb);
            if metric.error_count > 0 {
                println!("  Errors: {}", metric.error_count);
            }
            println!();
        }
    }
    
    /// Export report as JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    
    /// Save report to file
    pub async fn save_to_file(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        let json = self.to_json().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        tokio::fs::write(path, json).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::Database;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_benchmark_suite() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("bench_db");
        let database = Arc::new(Database::open(&db_path).unwrap());
        
        let config = BenchmarkConfig {
            operations_count: 100,
            concurrency: 2,
            warmup_operations: 10,
            ..Default::default()
        };
        
        let mut suite = BenchmarkSuite::new(database, config);
        let report = suite.run_full_suite().await.unwrap();
        
        assert!(!report.metrics.is_empty());
        assert!(report.metrics.iter().all(|m| m.total_operations > 0));
        
        // Print report for manual verification
        report.print_report();
    }
}
