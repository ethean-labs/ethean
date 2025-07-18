//! Performance benchmarking system for Beam Chain consensus
//!
//! Provides comprehensive performance metrics and benchmarking capabilities.

use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Performance metrics for different operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub attestation_processing: OperationMetrics,
    pub committee_calculation: OperationMetrics,
    pub signature_verification: OperationMetrics,
    pub validator_operations: OperationMetrics,
    pub memory_usage: MemoryMetrics,
}

/// Metrics for a specific operation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetrics {
    pub total_operations: u64,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub avg_duration: Duration,
    pub operations_per_second: f64,
    pub error_count: u64,
}

/// Memory usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub peak_memory_mb: f64,
    pub current_memory_mb: f64,
    pub cache_sizes: HashMap<String, usize>,
}

/// Performance monitor for tracking system performance
#[derive(Debug)]
pub struct PerformanceMonitor {
    start_time: Instant,
    operation_timings: HashMap<String, Vec<Duration>>,
    operation_errors: HashMap<String, u64>,
    memory_samples: Vec<f64>,
    cache_tracking: HashMap<String, usize>,
}

impl PerformanceMonitor {
    /// Create new performance monitor
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            operation_timings: HashMap::new(),
            operation_errors: HashMap::new(),
            memory_samples: Vec::new(),
            cache_tracking: HashMap::new(),
        }
    }
    
    /// Start timing an operation
    pub fn start_operation(&self, operation: &str) -> OperationTimer {
        OperationTimer::new(operation.to_string())
    }
    
    /// Record operation completion
    pub fn record_operation(&mut self, operation: &str, duration: Duration, success: bool) {
        // Record timing
        self.operation_timings
            .entry(operation.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
        
        // Record error if failed
        if !success {
            *self.operation_errors
                .entry(operation.to_string())
                .or_insert(0) += 1;
        }
    }
    
    /// Update cache size tracking
    pub fn update_cache_size(&mut self, cache_name: &str, size: usize) {
        self.cache_tracking.insert(cache_name.to_string(), size);
    }
    
    /// Sample current memory usage
    pub fn sample_memory(&mut self, memory_mb: f64) {
        self.memory_samples.push(memory_mb);
    }
    
    /// Get comprehensive performance metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            attestation_processing: self.calculate_operation_metrics("attestation_processing"),
            committee_calculation: self.calculate_operation_metrics("committee_calculation"),
            signature_verification: self.calculate_operation_metrics("signature_verification"),
            validator_operations: self.calculate_operation_metrics("validator_operations"),
            memory_usage: self.calculate_memory_metrics(),
        }
    }
    
    /// Calculate metrics for a specific operation
    fn calculate_operation_metrics(&self, operation: &str) -> OperationMetrics {
        let timings = self.operation_timings.get(operation).cloned().unwrap_or_default();
        let error_count = self.operation_errors.get(operation).cloned().unwrap_or(0);
        
        if timings.is_empty() {
            return OperationMetrics {
                total_operations: 0,
                total_duration: Duration::ZERO,
                min_duration: Duration::ZERO,
                max_duration: Duration::ZERO,
                avg_duration: Duration::ZERO,
                operations_per_second: 0.0,
                error_count,
            };
        }
        
        let total_duration: Duration = timings.iter().sum();
        let min_duration = *timings.iter().min().unwrap();
        let max_duration = *timings.iter().max().unwrap();
        let avg_duration = total_duration / timings.len() as u32;
        
        let total_seconds = self.start_time.elapsed().as_secs_f64();
        let operations_per_second = if total_seconds > 0.0 {
            timings.len() as f64 / total_seconds
        } else {
            0.0
        };
        
        OperationMetrics {
            total_operations: timings.len() as u64,
            total_duration,
            min_duration,
            max_duration,
            avg_duration,
            operations_per_second,
            error_count,
        }
    }
    
    /// Calculate memory metrics
    fn calculate_memory_metrics(&self) -> MemoryMetrics {
        let peak_memory_mb = self.memory_samples.iter().fold(0.0_f64, |a, &b| a.max(b));
        let current_memory_mb = self.memory_samples.last().cloned().unwrap_or(0.0);
        
        MemoryMetrics {
            peak_memory_mb,
            current_memory_mb,
            cache_sizes: self.cache_tracking.clone(),
        }
    }
    
    /// Reset all metrics
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.operation_timings.clear();
        self.operation_errors.clear();
        self.memory_samples.clear();
        self.cache_tracking.clear();
    }
    
    /// Generate performance report
    pub fn generate_report(&self) -> String {
        let metrics = self.get_metrics();
        let mut report = String::new();
        
        report.push_str("=== Beam Chain Performance Report ===\n\n");
        
        // Attestation processing
        report.push_str(&format!(
            "Attestation Processing:\n\
             - Total operations: {}\n\
             - Average duration: {:.2}ms\n\
             - Operations/second: {:.2}\n\
             - Error rate: {:.2}%\n\n",
            metrics.attestation_processing.total_operations,
            metrics.attestation_processing.avg_duration.as_millis(),
            metrics.attestation_processing.operations_per_second,
            if metrics.attestation_processing.total_operations > 0 {
                (metrics.attestation_processing.error_count as f64 / 
                 metrics.attestation_processing.total_operations as f64) * 100.0
            } else { 0.0 }
        ));
        
        // Committee calculation
        report.push_str(&format!(
            "Committee Calculation:\n\
             - Total operations: {}\n\
             - Average duration: {:.2}ms\n\
             - Operations/second: {:.2}\n\n",
            metrics.committee_calculation.total_operations,
            metrics.committee_calculation.avg_duration.as_millis(),
            metrics.committee_calculation.operations_per_second,
        ));
        
        // Signature verification
        report.push_str(&format!(
            "Signature Verification:\n\
             - Total operations: {}\n\
             - Average duration: {:.2}ms\n\
             - Operations/second: {:.2}\n\n",
            metrics.signature_verification.total_operations,
            metrics.signature_verification.avg_duration.as_millis(),
            metrics.signature_verification.operations_per_second,
        ));
        
        // Memory usage
        report.push_str(&format!(
            "Memory Usage:\n\
             - Peak memory: {:.2} MB\n\
             - Current memory: {:.2} MB\n\
             - Cache sizes: {:?}\n\n",
            metrics.memory_usage.peak_memory_mb,
            metrics.memory_usage.current_memory_mb,
            metrics.memory_usage.cache_sizes,
        ));
        
        report
    }
}

/// Timer for measuring operation duration
pub struct OperationTimer {
    operation: String,
    start_time: Instant,
}

impl OperationTimer {
    /// Create new operation timer
    pub fn new(operation: String) -> Self {
        Self {
            operation,
            start_time: Instant::now(),
        }
    }
    
    /// Finish timing and return duration
    pub fn finish(self) -> (String, Duration) {
        (self.operation, self.start_time.elapsed())
    }
}

/// Benchmark runner for performance testing
pub struct BenchmarkRunner {
    monitor: PerformanceMonitor,
}

impl BenchmarkRunner {
    /// Create new benchmark runner
    pub fn new() -> Self {
        Self {
            monitor: PerformanceMonitor::new(),
        }
    }
    
    /// Run attestation processing benchmark
    pub fn benchmark_attestation_processing(&mut self, iterations: u32) -> Duration {
        let start = Instant::now();
        
        for i in 0..iterations {
            let timer = self.monitor.start_operation("attestation_processing");
            
            // Simulate attestation processing work
            std::thread::sleep(Duration::from_micros(100));
            
            let (operation, duration) = timer.finish();
            self.monitor.record_operation(&operation, duration, i % 100 != 0); // 1% error rate
        }
        
        start.elapsed()
    }
    
    /// Run committee calculation benchmark
    pub fn benchmark_committee_calculation(&mut self, iterations: u32) -> Duration {
        let start = Instant::now();
        
        for _i in 0..iterations {
            let timer = self.monitor.start_operation("committee_calculation");
            
            // Simulate committee calculation work
            std::thread::sleep(Duration::from_micros(50));
            
            let (operation, duration) = timer.finish();
            self.monitor.record_operation(&operation, duration, true);
        }
        
        start.elapsed()
    }
    
    /// Run signature verification benchmark
    pub fn benchmark_signature_verification(&mut self, iterations: u32) -> Duration {
        let start = Instant::now();
        
        for i in 0..iterations {
            let timer = self.monitor.start_operation("signature_verification");
            
            // Simulate signature verification work
            std::thread::sleep(Duration::from_micros(200));
            
            let (operation, duration) = timer.finish();
            self.monitor.record_operation(&operation, duration, i % 50 != 0); // 2% error rate
        }
        
        start.elapsed()
    }
    
    /// Run comprehensive benchmark suite
    pub fn run_full_benchmark(&mut self) -> String {
        println!("Running comprehensive Beam Chain performance benchmark...");
        
        // Reset monitor
        self.monitor.reset();
        
        // Run benchmarks
        println!("Benchmarking attestation processing...");
        self.benchmark_attestation_processing(1000);
        
        println!("Benchmarking committee calculation...");
        self.benchmark_committee_calculation(500);
        
        println!("Benchmarking signature verification...");
        self.benchmark_signature_verification(300);
        
        // Simulate memory sampling
        for i in 1..=10 {
            self.monitor.sample_memory(100.0 + i as f64 * 5.0);
        }
        
        // Update cache sizes
        self.monitor.update_cache_size("committee_cache", 150);
        self.monitor.update_cache_size("signature_cache", 200);
        self.monitor.update_cache_size("validator_cache", 1000);
        
        // Generate report
        self.monitor.generate_report()
    }
    
    /// Get current performance metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.monitor.get_metrics()
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for BenchmarkRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        let metrics = monitor.get_metrics();
        
        assert_eq!(metrics.attestation_processing.total_operations, 0);
        assert_eq!(metrics.committee_calculation.total_operations, 0);
    }
    
    #[test]
    fn test_operation_timer() {
        let timer = OperationTimer::new("test_operation".to_string());
        std::thread::sleep(Duration::from_millis(1));
        let (operation, duration) = timer.finish();
        
        assert_eq!(operation, "test_operation");
        assert!(duration >= Duration::from_millis(1));
    }
    
    #[test]
    fn test_operation_recording() {
        let mut monitor = PerformanceMonitor::new();
        
        monitor.record_operation("test", Duration::from_millis(10), true);
        monitor.record_operation("test", Duration::from_millis(20), false);
        
        let metrics = monitor.calculate_operation_metrics("test");
        assert_eq!(metrics.total_operations, 2);
        assert_eq!(metrics.error_count, 1);
    }
    
    #[test]
    fn test_benchmark_runner() {
        let mut runner = BenchmarkRunner::new();
        
        // Run small benchmark
        runner.benchmark_attestation_processing(10);
        
        let metrics = runner.get_metrics();
        assert!(metrics.attestation_processing.total_operations > 0);
    }
    
    #[test]
    fn test_cache_tracking() {
        let mut monitor = PerformanceMonitor::new();
        
        monitor.update_cache_size("test_cache", 100);
        monitor.update_cache_size("another_cache", 200);
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.memory_usage.cache_sizes.get("test_cache"), Some(&100));
        assert_eq!(metrics.memory_usage.cache_sizes.get("another_cache"), Some(&200));
    }
    
    #[test]
    fn test_memory_sampling() {
        let mut monitor = PerformanceMonitor::new();
        
        monitor.sample_memory(100.0);
        monitor.sample_memory(150.0);
        monitor.sample_memory(120.0);
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.memory_usage.peak_memory_mb, 150.0);
        assert_eq!(metrics.memory_usage.current_memory_mb, 120.0);
    }
}
