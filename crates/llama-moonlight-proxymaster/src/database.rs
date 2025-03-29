//! Database module.
//! Handles database initialization and operations.

use crate::models::Proxy;
use log::{debug, error, info};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Pool, Sqlite, SqlitePool};
use std::time::Duration;
use uuid::Uuid;

/// Initializes the database and returns a connection pool.
pub async fn init_db(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    // Create database if it doesn't exist
    if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
        info!("Creating database at {}", database_url);
        Sqlite::create_database(database_url).await?;
    }
    
    // Connect to the database
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await?;
    
    // Create tables if they don't exist
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
    .execute(&pool)
    .await?;
    
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
    .execute(&pool)
    .await?;
    
    // Create index on ip and port
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_proxies_ip_port ON proxies(ip, port)
        "#,
    )
    .execute(&pool)
    .await?;
    
    Ok(pool)
}

/// Saves a proxy to the database, updating if it already exists.
pub async fn save_proxy(pool: &SqlitePool, proxy: &Proxy) -> Result<(), sqlx::Error> {
    // Start a transaction
    let mut tx = pool.begin().await?;
    
    // Insert or update proxy
    sqlx::query(
        r#"
        INSERT INTO proxies (
            id, ip, port, country, anonymity, https, last_checked, 
            response_time, weight, success_rate
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            ip = excluded.ip,
            port = excluded.port,
            country = excluded.country,
            anonymity = excluded.anonymity,
            https = excluded.https,
            last_checked = excluded.last_checked,
            response_time = excluded.response_time,
            weight = excluded.weight,
            success_rate = excluded.success_rate
        "#,
    )
    .bind(&proxy.id.to_string())
    .bind(&proxy.ip)
    .bind(proxy.port as i64)
    .bind(&proxy.country)
    .bind(&proxy.anonymity)
    .bind(proxy.https as i64)
    .bind(proxy.last_checked.as_ref().map(|d| d.to_rfc3339()))
    .bind(proxy.response_time)
    .bind(proxy.weight)
    .bind(proxy.success_rate)
    .execute(&mut *tx)
    .await?;
    
    // Delete existing protocols for this proxy
    sqlx::query(
        r#"
        DELETE FROM proxy_protocols WHERE proxy_id = ?
        "#,
    )
    .bind(&proxy.id.to_string())
    .execute(&mut *tx)
    .await?;
    
    // Insert protocols
    for protocol in &proxy.protocols {
        sqlx::query(
            r#"
            INSERT INTO proxy_protocols (proxy_id, protocol) VALUES (?, ?)
            "#,
        )
        .bind(&proxy.id.to_string())
        .bind(protocol)
        .execute(&mut *tx)
        .await?;
    }
    
    // Commit transaction
    tx.commit().await?;
    
    debug!("Saved proxy {} to database", proxy.as_str());
    Ok(())
}

/// Loads all proxies from the database.
pub async fn load_proxies(pool: &SqlitePool) -> Result<Vec<Proxy>, sqlx::Error> {
    // Query proxies
    let proxy_rows = sqlx::query!(
        r#"
        SELECT 
            id, ip, port, country, anonymity, https, last_checked, 
            response_time, weight, success_rate
        FROM proxies
        "#
    )
    .fetch_all(pool)
    .await?;
    
    let mut proxies = Vec::with_capacity(proxy_rows.len());
    
    // Assemble proxies with their protocols
    for row in proxy_rows {
        let id = Uuid::parse_str(&row.id).unwrap_or_else(|_| Uuid::new_v4());
        
        // Get protocols for this proxy
        let protocol_rows = sqlx::query!(
            r#"
            SELECT protocol FROM proxy_protocols WHERE proxy_id = ?
            "#,
            row.id
        )
        .fetch_all(pool)
        .await?;
        
        let protocols = protocol_rows.iter().map(|p| p.protocol.clone()).collect();
        
        // Parse last_checked if present
        let last_checked = row.last_checked.and_then(|date_str| {
            chrono::DateTime::parse_from_rfc3339(&date_str)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        });
        
        let proxy = Proxy {
            id,
            ip: row.ip.clone(),
            port: row.port as u16,
            country: row.country.clone(),
            anonymity: row.anonymity.clone(),
            https: row.https != 0,
            protocols,
            last_checked,
            response_time: row.response_time,
            weight: row.weight,
            success_rate: row.success_rate,
        };
        
        proxies.push(proxy);
    }
    
    Ok(proxies)
}

/// Deletes a proxy from the database.
pub async fn delete_proxy(pool: &SqlitePool, id: &Uuid) -> Result<bool, sqlx::Error> {
    // Start a transaction
    let mut tx = pool.begin().await?;
    
    // Delete protocols first (due to foreign key constraint)
    sqlx::query(
        r#"
        DELETE FROM proxy_protocols WHERE proxy_id = ?
        "#,
    )
    .bind(id.to_string())
    .execute(&mut *tx)
    .await?;
    
    // Delete proxy
    let result = sqlx::query(
        r#"
        DELETE FROM proxies WHERE id = ?
        "#,
    )
    .bind(id.to_string())
    .execute(&mut *tx)
    .await?;
    
    // Commit transaction
    tx.commit().await?;
    
    // Return whether a row was affected
    Ok(result.rows_affected() > 0)
}

/// Gets a count of proxies in the database.
pub async fn count_proxies(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT COUNT(*) as count FROM proxies
        "#
    )
    .fetch_one(pool)
    .await?;
    
    Ok(row.count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_database_operations() {
        // Create a temporary directory for the test database
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db.sqlite");
        let db_url = format!("sqlite:{}", db_path.display());
        
        // Initialize database
        let pool = init_db(&db_url).await.unwrap();
        
        // Create a test proxy
        let mut proxy = Proxy::new("192.168.1.1".to_string(), 8080, true);
        proxy.protocols = vec!["http".to_string(), "socks5".to_string()];
        
        // Save proxy
        save_proxy(&pool, &proxy).await.unwrap();
        
        // Load proxies
        let loaded_proxies = load_proxies(&pool).await.unwrap();
        assert_eq!(loaded_proxies.len(), 1);
        
        let loaded_proxy = &loaded_proxies[0];
        assert_eq!(loaded_proxy.ip, proxy.ip);
        assert_eq!(loaded_proxy.port, proxy.port);
        assert_eq!(loaded_proxy.https, proxy.https);
        assert_eq!(loaded_proxy.protocols.len(), 2);
        
        // Count proxies
        let count = count_proxies(&pool).await.unwrap();
        assert_eq!(count, 1);
        
        // Delete proxy
        let deleted = delete_proxy(&pool, &proxy.id).await.unwrap();
        assert!(deleted);
        
        // Verify deletion
        let count_after = count_proxies(&pool).await.unwrap();
        assert_eq!(count_after, 0);
    }
} 