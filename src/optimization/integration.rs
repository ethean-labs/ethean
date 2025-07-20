//! Integration module for optimization system
//!
//! This module handles the integration of the advanced optimization system
//! with the rest of the Panro Ethereum Beacon Chain client.

use crate::optimization::{
    AdvancedOptimizationSystem,
    OptimizationSystemConfig,
    OptimizationSystemError,
};

use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

/// Optimization system integration manager
pub struct OptimizationIntegration {
    /// The optimization system instance
    optimization_system: Arc<RwLock<AdvancedOptimizationSystem>>,
    /// Integration configuration
    config: IntegrationConfig,
    /// Current integration status
    status: Arc<RwLock<IntegrationStatus>>,
}

#[derive(Debug, Clone)]
pub struct IntegrationConfig {
    /// Auto-start optimization system
    pub auto_start: bool,
    /// Auto-optimization interval
    pub auto_optimization_interval: Duration,
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Enable auto-recommendations
    pub enable_auto_recommendations: bool,
    /// Maximum concurrent optimizations
    pub max_concurrent_optimizations: usize,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            auto_start: true,
            auto_optimization_interval: Duration::from_secs(1800), // 30 minutes
            enable_performance_monitoring: true,
            enable_auto_recommendations: true,
            max_concurrent_optimizations: 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IntegrationStatus {
    /// Whether the optimization system is active
    pub active: bool,
    /// Last optimization timestamp
    pub last_optimization: Option<std::time::Instant>,
    /// Number of optimizations performed
    pub optimizations_performed: u64,
    /// Average optimization improvement
    pub average_improvement: f64,
    /// Current system performance score
    pub current_performance_score: f64,
}

impl OptimizationIntegration {
    /// Create new optimization integration
    pub async fn new(
        optimization_config: OptimizationSystemConfig,
        integration_config: IntegrationConfig,
    ) -> Result<Self, OptimizationSystemError> {
        let optimization_system = AdvancedOptimizationSystem::new(optimization_config).await?;
        
        let integration = Self {
            optimization_system: Arc::new(RwLock::new(optimization_system)),
            config: integration_config,
            status: Arc::new(RwLock::new(IntegrationStatus {
                active: false,
                last_optimization: None,
                optimizations_performed: 0,
                average_improvement: 0.0,
                current_performance_score: 0.0,
            })),
        };

        Ok(integration)
    }

    /// Initialize and start the optimization integration
    pub async fn initialize(&mut self) -> Result<(), OptimizationSystemError> {
        tracing::info!("Initializing optimization system integration");

        // Start the optimization system
        {
            let mut opt_system = self.optimization_system.write().await;
            opt_system.start().await?;
        }

        // Update status
        {
            let mut status = self.status.write().await;
            status.active = true;
        }

        // Start background tasks if auto-start is enabled
        if self.config.auto_start {
            self.start_background_tasks().await?;
        }

        tracing::info!("Optimization system integration initialized successfully");
        Ok(())
    }

    /// Get current integration status
    pub async fn get_status(&self) -> IntegrationStatus {
        self.status.read().await.clone()
    }

    /// Run manual optimization
    pub async fn run_optimization(&self) -> Result<(), OptimizationSystemError> {
        tracing::info!("Running manual optimization");

        let optimization_result = {
            let mut opt_system = self.optimization_system.write().await;
            opt_system.optimize().await?
        };

        // Update status
        {
            let mut status = self.status.write().await;
            status.last_optimization = Some(std::time::Instant::now());
            status.optimizations_performed += 1;
            
            // Update average improvement
            let new_avg = (status.average_improvement * (status.optimizations_performed - 1) as f64 + 
                          optimization_result.performance_improvement) / status.optimizations_performed as f64;
            status.average_improvement = new_avg;
        }

        tracing::info!(
            "Manual optimization completed with {:.2}% improvement",
            optimization_result.performance_improvement * 100.0
        );

        Ok(())
    }

    /// Get optimization recommendations
    pub async fn get_recommendations(&self) -> Result<Vec<crate::optimization::SystemOptimizationRecommendation>, OptimizationSystemError> {
        let mut opt_system = self.optimization_system.write().await;
        opt_system.get_recommendations().await
    }

    /// Apply specific recommendation
    pub async fn apply_recommendation(&self, recommendation_id: &str) -> Result<(), OptimizationSystemError> {
        tracing::info!("Applying recommendation: {}", recommendation_id);

        let mut opt_system = self.optimization_system.write().await;
        let result = opt_system.apply_recommendation(recommendation_id).await?;

        if result.success {
            tracing::info!(
                "Recommendation {} applied successfully with {:.2}% improvement",
                recommendation_id,
                result.performance_improvement * 100.0
            );
        } else {
            tracing::warn!("Failed to apply recommendation: {}", recommendation_id);
        }

        Ok(())
    }

    /// Get comprehensive system status
    pub async fn get_system_status(&self) -> crate::optimization::SystemStatusReport {
        let opt_system = self.optimization_system.read().await;
        opt_system.get_system_status().await
    }

    /// Enable or disable auto-optimization
    pub async fn set_auto_optimization(&mut self, enabled: bool) -> Result<(), OptimizationSystemError> {
        self.config.enable_auto_recommendations = enabled;
        
        if enabled && !self.config.auto_start {
            self.start_background_tasks().await?;
        }

        tracing::info!("Auto-optimization {}", if enabled { "enabled" } else { "disabled" });
        Ok(())
    }

    /// Shutdown the optimization integration
    pub async fn shutdown(&mut self) -> Result<(), OptimizationSystemError> {
        tracing::info!("Shutting down optimization system integration");

        // Update status
        {
            let mut status = self.status.write().await;
            status.active = false;
        }

        // Stop background tasks
        self.stop_background_tasks().await?;

        tracing::info!("Optimization system integration shut down successfully");
        Ok(())
    }

    // Private implementation methods
    async fn start_background_tasks(&self) -> Result<(), OptimizationSystemError> {
        if self.config.enable_performance_monitoring {
            self.start_performance_monitoring_task().await?;
        }

        if self.config.enable_auto_recommendations {
            self.start_auto_optimization_task().await?;
        }

        Ok(())
    }

    async fn start_performance_monitoring_task(&self) -> Result<(), OptimizationSystemError> {
        tracing::info!("Starting performance monitoring background task");
        
        let optimization_system = Arc::clone(&self.optimization_system);
        let status = Arc::clone(&self.status);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                // Check if still active
                {
                    let current_status = status.read().await;
                    if !current_status.active {
                        break;
                    }
                }

                // Collect and update performance metrics
                if let Ok(opt_system) = optimization_system.try_read() {
                    let system_status = opt_system.get_system_status().await;
                    
                    let mut status_write = status.write().await;
                    status_write.current_performance_score = system_status.overall_performance_score;
                }
            }
            
            tracing::info!("Performance monitoring task stopped");
        });

        Ok(())
    }

    async fn start_auto_optimization_task(&self) -> Result<(), OptimizationSystemError> {
        tracing::info!("Starting auto-optimization background task");
        
        let optimization_system = Arc::clone(&self.optimization_system);
        let status = Arc::clone(&self.status);
        let interval_duration = self.config.auto_optimization_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval_duration);
            
            loop {
                interval.tick().await;
                
                // Check if still active
                {
                    let current_status = status.read().await;
                    if !current_status.active {
                        break;
                    }
                }

                // Run auto-optimization
                if let Ok(mut opt_system) = optimization_system.try_write() {
                    match opt_system.optimize().await {
                        Ok(result) => {
                            tracing::info!(
                                "Auto-optimization completed with {:.2}% improvement",
                                result.performance_improvement * 100.0
                            );
                            
                            // Update status
                            if let Ok(mut status_write) = status.try_write() {
                                status_write.last_optimization = Some(std::time::Instant::now());
                                status_write.optimizations_performed += 1;
                                
                                let new_avg = (status_write.average_improvement * (status_write.optimizations_performed - 1) as f64 + 
                                              result.performance_improvement) / status_write.optimizations_performed as f64;
                                status_write.average_improvement = new_avg;
                            }
                        }
                        Err(e) => {
                            tracing::error!("Auto-optimization failed: {}", e);
                        }
                    }
                }
            }
            
            tracing::info!("Auto-optimization task stopped");
        });

        Ok(())
    }

    async fn stop_background_tasks(&self) -> Result<(), OptimizationSystemError> {
        tracing::info!("Stopping background optimization tasks");
        // Tasks will stop automatically when status.active becomes false
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_integration_creation() {
        let opt_config = OptimizationSystemConfig::default();
        let int_config = IntegrationConfig::default();
        
        let integration = OptimizationIntegration::new(opt_config, int_config).await;
        assert!(integration.is_ok());
    }

    #[tokio::test]
    async fn test_integration_status() {
        let opt_config = OptimizationSystemConfig::default();
        let int_config = IntegrationConfig::default();
        
        let integration = OptimizationIntegration::new(opt_config, int_config).await.unwrap();
        let status = integration.get_status().await;
        
        assert!(!status.active);
        assert_eq!(status.optimizations_performed, 0);
    }

    #[test]
    fn test_integration_config_default() {
        let config = IntegrationConfig::default();
        assert!(config.auto_start);
        assert!(config.enable_performance_monitoring);
        assert!(config.enable_auto_recommendations);
        assert_eq!(config.max_concurrent_optimizations, 3);
    }
}
