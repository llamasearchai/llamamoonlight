//! Pool module.
//! Manages a pool of proxies for rotation and validation.

use crate::database::{delete_proxy, load_proxies, save_proxy};
use crate::models::{Proxy, SelectionStrategy};
use crate::validator::{validate_proxy, ValidatorConfig};
use log::{debug, error, info, warn};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Configuration for the proxy pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Selection strategy for proxy rotation.
    pub strategy: SelectionStrategy,
    
    /// Minimum weight for a proxy to be considered viable.
    pub min_weight: f32,
    
    /// Minimum success rate for a proxy to be considered reliable.
    pub min_success_rate: f32,
    
    /// Whether to automatically remove failed proxies.
    pub auto_remove_failed: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            strategy: SelectionStrategy::Weighted,
            min_weight: 0.5,
            min_success_rate: 0.0,
            auto_remove_failed: true,
        }
    }
}

/// Proxy pool for managing and rotating proxies.
#[derive(Clone)]
pub struct ProxyPool {
    /// SQLite database pool.
    pub db: SqlitePool,
    
    /// In-memory collection of proxies.
    proxies: Arc<RwLock<Vec<Proxy>>>,
    
    /// Pool configuration.
    config: PoolConfig,
    
    /// Validator configuration.
    validator_config: ValidatorConfig,
    
    /// Current index for round-robin selection.
    current_index: Arc<RwLock<usize>>,
}

impl ProxyPool {
    /// Creates a new proxy pool.
    pub fn new(db: SqlitePool) -> Self {
        Self::with_config(db, PoolConfig::default(), ValidatorConfig::default())
    }
    
    /// Creates a new proxy pool with custom configuration.
    pub fn with_config(
        db: SqlitePool,
        config: PoolConfig,
        validator_config: ValidatorConfig,
    ) -> Self {
        Self {
            db,
            proxies: Arc::new(RwLock::new(Vec::new())),
            config,
            validator_config,
            current_index: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Initializes the pool by loading proxies from the database.
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Initializing proxy pool");
        let proxies = load_proxies(&self.db).await?;
        info!("Loaded {} proxies from database", proxies.len());
        
        let mut pool = self.proxies.write().await;
        *pool = proxies;
        
        Ok(())
    }
    
    /// Adds new proxies to the pool.
    pub async fn add_proxies(&self, new_proxies: Vec<Proxy>) -> usize {
        let mut added = 0;
        let mut pool = self.proxies.write().await;
        
        // Add new proxies that don't already exist
        for proxy in new_proxies {
            // Skip if it already exists
            if pool.iter().any(|p| p.ip == proxy.ip && p.port == proxy.port) {
                continue;
            }
            
            // Save to database
            if let Err(e) = save_proxy(&self.db, &proxy).await {
                error!("Failed to save proxy {}: {}", proxy.as_str(), e);
                continue;
            }
            
            // Add to memory pool
            pool.push(proxy);
            added += 1;
        }
        
        info!("Added {} new proxies to pool (total: {})", added, pool.len());
        added
    }
    
    /// Gets a proxy using the configured selection strategy.
    pub async fn get_proxy(&self) -> Option<Proxy> {
        let pool = self.proxies.read().await;
        if pool.is_empty() {
            return None;
        }
        
        match self.config.strategy {
            SelectionStrategy::Random => self.get_random_proxy(&pool),
            SelectionStrategy::Weighted => self.get_weighted_proxy(&pool),
            SelectionStrategy::RoundRobin => self.get_round_robin_proxy(&pool).await,
            SelectionStrategy::Fastest => self.get_fastest_proxy(&pool),
        }
    }
    
    /// Gets a random proxy from the pool.
    fn get_random_proxy(&self, pool: &[Proxy]) -> Option<Proxy> {
        if pool.is_empty() {
            return None;
        }
        
        let mut rng = thread_rng();
        let index = rng.gen_range(0..pool.len());
        Some(pool[index].clone())
    }
    
    /// Gets a proxy by weighted random selection.
    fn get_weighted_proxy(&self, pool: &[Proxy]) -> Option<Proxy> {
        if pool.is_empty() {
            return None;
        }
        
        // Calculate total weight
        let total_weight: f32 = pool.iter()
            .filter(|p| p.weight >= self.config.min_weight)
            .map(|p| p.weight)
            .sum();
            
        if total_weight <= 0.0 {
            // Fall back to random selection if all weights are zero
            return self.get_random_proxy(pool);
        }
        
        // Weighted random selection
        let mut rng = thread_rng();
        let mut r = rng.gen_range(0.0..total_weight);
        
        for proxy in pool.iter().filter(|p| p.weight >= self.config.min_weight) {
            r -= proxy.weight;
            if r <= 0.0 {
                return Some(proxy.clone());
            }
        }
        
        // Fallback
        Some(pool[0].clone())
    }
    
    /// Gets a proxy using round-robin selection.
    async fn get_round_robin_proxy(&self, pool: &[Proxy]) -> Option<Proxy> {
        if pool.is_empty() {
            return None;
        }
        
        let mut index = self.current_index.write().await;
        let proxy = pool[*index].clone();
        
        *index = (*index + 1) % pool.len();
        
        Some(proxy)
    }
    
    /// Gets the fastest proxy from the pool.
    fn get_fastest_proxy(&self, pool: &[Proxy]) -> Option<Proxy> {
        pool.iter()
            .filter(|p| p.weight >= self.config.min_weight)
            .max_by(|a, b| a.weight.partial_cmp(&b.weight).unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
    }
    
    /// Removes a proxy from the pool.
    pub async fn remove_proxy(&self, id: &Uuid) -> bool {
        // Remove from memory pool
        {
            let mut pool = self.proxies.write().await;
            let len_before = pool.len();
            pool.retain(|p| p.id != *id);
            if pool.len() == len_before {
                return false; // Not found in memory
            }
        }
        
        // Remove from database
        match delete_proxy(&self.db, id).await {
            Ok(deleted) => {
                if deleted {
                    debug!("Proxy {} removed from database", id);
                    true
                } else {
                    warn!("Proxy {} was in memory but not in database", id);
                    false
                }
            },
            Err(e) => {
                error!("Failed to delete proxy {} from database: {}", id, e);
                false
            }
        }
    }
    
    /// Validates all proxies in the pool.
    pub async fn validate_all(&self, concurrency: usize) {
        info!("Validating all proxies with concurrency {}", concurrency);
        
        // Get all proxies
        let proxies = {
            let pool = self.proxies.read().await;
            pool.clone()
        };
        
        if proxies.is_empty() {
            info!("No proxies to validate");
            return;
        }
        
        // Process in batches to control concurrency
        let batch_size = concurrency.min(100).max(1); // Between 1 and 100
        let chunks = proxies.chunks(batch_size);
        let total_chunks = (proxies.len() + batch_size - 1) / batch_size;
        
        info!("Starting validation of {} proxies in {} batches", proxies.len(), total_chunks);
        
        let mut chunk_index = 0;
        let mut updated_proxies = Vec::new();
        let mut failed_proxies = Vec::new();
        
        for chunk in chunks {
            chunk_index += 1;
            info!("Validating batch {}/{}", chunk_index, total_chunks);
            
            let mut tasks = Vec::with_capacity(chunk.len());
            
            for proxy in chunk {
                let proxy_clone = proxy.clone();
                let config = self.validator_config.clone();
                
                tasks.push(tokio::spawn(async move {
                    let mut proxy = proxy_clone;
                    let result = validate_proxy(&mut proxy, &config).await;
                    (proxy, result.is_working)
                }));
            }
            
            for task in tasks {
                match task.await {
                    Ok((mut proxy, is_working)) => {
                        // Update the proxy in the database
                        if let Err(e) = save_proxy(&self.db, &proxy).await {
                            error!("Failed to update proxy {}: {}", proxy.as_str(), e);
                        }
                        
                        if is_working {
                            updated_proxies.push(proxy);
                        } else if self.config.auto_remove_failed {
                            failed_proxies.push(proxy.id);
                        } else {
                            // Still keep it in memory, but reduce its weight
                            proxy.weight *= 0.5;
                            if proxy.weight >= self.config.min_weight {
                                updated_proxies.push(proxy);
                            } else {
                                failed_proxies.push(proxy.id);
                            }
                        }
                    },
                    Err(e) => {
                        error!("Task error during proxy validation: {}", e);
                    }
                }
            }
        }
        
        // Update the in-memory pool
        {
            let mut pool = self.proxies.write().await;
            
            // Replace with updated proxies
            *pool = updated_proxies;
            
            info!("Validation complete: {} proxies remain in pool", pool.len());
        }
        
        // Remove failed proxies from database if auto_remove_failed
        if self.config.auto_remove_failed {
            info!("Removing {} failed proxies from database", failed_proxies.len());
            
            for id in failed_proxies {
                if let Err(e) = delete_proxy(&self.db, &id).await {
                    error!("Failed to delete proxy {} from database: {}", id, e);
                }
            }
        }
    }
    
    /// Gets the count of proxies in the pool.
    pub async fn count(&self) -> usize {
        self.proxies.read().await.len()
    }
    
    /// Gets all proxies in the pool.
    pub async fn get_all(&self) -> Vec<Proxy> {
        self.proxies.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::migrate::MigrateDatabase;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_pool_initialization() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());
        
        // Create database
        sqlx::Sqlite::create_database(&db_url).await.unwrap();
        
        // Initialize database
        let db_pool = SqlitePool::connect(&db_url).await.unwrap();
        
        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS proxies (
                id TEXT PRIMARY KEY,
                ip TEXT NOT NULL,
                port INTEGER NOT NULL,
                country TEXT,
                anonymity TEXT,
                https INTEGER NOT NULL,
                last_checked TEXT,
                response_time INTEGER,
                weight REAL NOT NULL,
                success_rate REAL NOT NULL
            )
            "#,
        )
        .execute(&db_pool)
        .await
        .unwrap();
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS proxy_protocols (
                proxy_id TEXT NOT NULL,
                protocol TEXT NOT NULL,
                PRIMARY KEY (proxy_id, protocol),
                FOREIGN KEY (proxy_id) REFERENCES proxies(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&db_pool)
        .await
        .unwrap();
        
        // Create proxy pool
        let pool = ProxyPool::new(db_pool);
        
        // Add a test proxy
        let proxy = Proxy::new("127.0.0.1".to_string(), 8080, true);
        pool.add_proxies(vec![proxy]).await;
        
        // Check count
        assert_eq!(pool.count().await, 1);
    }
    
    #[tokio::test]
    async fn test_proxy_selection_strategies() {
        // Create in-memory database
        let db_pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        
        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS proxies (
                id TEXT PRIMARY KEY,
                ip TEXT NOT NULL,
                port INTEGER NOT NULL,
                country TEXT,
                anonymity TEXT,
                https INTEGER NOT NULL,
                last_checked TEXT,
                response_time INTEGER,
                weight REAL NOT NULL,
                success_rate REAL NOT NULL
            )
            "#,
        )
        .execute(&db_pool)
        .await
        .unwrap();
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS proxy_protocols (
                proxy_id TEXT NOT NULL,
                protocol TEXT NOT NULL,
                PRIMARY KEY (proxy_id, protocol),
                FOREIGN KEY (proxy_id) REFERENCES proxies(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&db_pool)
        .await
        .unwrap();
        
        // Create proxy pool with random selection
        let config = PoolConfig {
            strategy: SelectionStrategy::Random,
            ..Default::default()
        };
        
        let pool = ProxyPool::with_config(db_pool, config, ValidatorConfig::default());
        
        // Add test proxies
        let mut proxies = Vec::new();
        for i in 1..5 {
            let mut proxy = Proxy::new(format!("192.168.1.{}", i), 8080, true);
            proxy.weight = i as f32;
            proxies.push(proxy);
        }
        
        pool.add_proxies(proxies).await;
        
        // Test random selection
        let random_proxy = pool.get_proxy().await;
        assert!(random_proxy.is_some());
        
        // Switch to weighted selection
        let mut config = pool.config.clone();
        config.strategy = SelectionStrategy::Weighted;
        
        let weighted_pool = ProxyPool::with_config(pool.db.clone(), config, ValidatorConfig::default());
        weighted_pool.initialize().await.unwrap();
        
        // Test weighted selection
        let weighted_proxy = weighted_pool.get_proxy().await;
        assert!(weighted_proxy.is_some());
    }
} 