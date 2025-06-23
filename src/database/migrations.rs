// Database migration management utilities
// Provides additional functionality for managing database migrations

use crate::utils::Result;
use sqlx::PgPool;
use tracing::{info, warn};

/// Migration manager for database schema evolution
pub struct MigrationManager {
    pool: PgPool,
}

impl MigrationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Run all pending migrations
    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations...");
        
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await?;
            
        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Get migration status information
    pub async fn status(&self) -> Result<MigrationStatus> {
        info!("Checking migration status...");
        
        // Check if migrations table exists
        let migrations_exist = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (
                SELECT FROM information_schema.tables 
                WHERE table_schema = 'public' 
                AND table_name = '_sqlx_migrations'
            )"
        )
        .fetch_one(&self.pool)
        .await?;

        if !migrations_exist {
            return Ok(MigrationStatus {
                applied_migrations: 0,
                pending_migrations: self.count_migration_files(),
                last_migration: None,
                database_ready: false,
            });
        }

        // Get applied migrations count
        let applied_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM _sqlx_migrations WHERE success = true"
        )
        .fetch_one(&self.pool)
        .await?;

        // Get last applied migration
        let last_migration = sqlx::query_scalar::<_, Option<String>>(
            "SELECT description FROM _sqlx_migrations 
             WHERE success = true 
             ORDER BY installed_on DESC 
             LIMIT 1"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(MigrationStatus {
            applied_migrations: applied_count as usize,
            pending_migrations: self.count_migration_files().saturating_sub(applied_count as usize),
            last_migration,
            database_ready: applied_count > 0,
        })
    }

    /// Validate database schema integrity
    pub async fn validate_schema(&self) -> Result<SchemaValidation> {
        info!("Validating database schema...");
        
        let mut validation = SchemaValidation {
            valid: true,
            issues: Vec::new(),
        };

        // Check if required tables exist
        let required_tables = [
            "instruments",
            "market_ticks", 
            "ai_predictions",
            "microstructure_analysis",
            "trading_signals",
            "trade_executions",
            "order_book_snapshots",
            "risk_metrics",
        ];

        for table in required_tables {
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS (
                    SELECT FROM information_schema.tables 
                    WHERE table_schema = 'public' 
                    AND table_name = $1
                )"
            )
            .bind(table)
            .fetch_one(&self.pool)
            .await?;

            if !exists {
                validation.valid = false;
                validation.issues.push(format!("Missing required table: {}", table));
            }
        }

        // Check if hypertables are properly configured
        let hypertables = sqlx::query_scalar::<_, String>(
            "SELECT table_name FROM timescaledb_information.hypertables
             WHERE schema_name = 'public'"
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let expected_hypertables = [
            "market_ticks",
            "ai_predictions", 
            "microstructure_analysis",
            "trading_signals",
            "trade_executions",
            "order_book_snapshots",
            "risk_metrics",
        ];

        for table in expected_hypertables {
            if !hypertables.contains(&table.to_string()) {
                validation.valid = false;
                validation.issues.push(format!("Table {} is not a hypertable", table));
            }
        }

        if validation.valid {
            info!("Database schema validation passed");
        } else {
            warn!("Database schema validation failed: {:?}", validation.issues);
        }

        Ok(validation)
    }

    /// Reset database (WARNING: This will drop all data)
    pub async fn reset(&self) -> Result<()> {
        warn!("DANGER: Resetting database - all data will be lost!");
        
        // Drop all tables in reverse dependency order
        let drop_queries = [
            "DROP TABLE IF EXISTS risk_metrics CASCADE",
            "DROP TABLE IF EXISTS trade_executions CASCADE", 
            "DROP TABLE IF EXISTS trading_signals CASCADE",
            "DROP TABLE IF EXISTS microstructure_analysis CASCADE",
            "DROP TABLE IF EXISTS ai_predictions CASCADE",
            "DROP TABLE IF EXISTS order_book_snapshots CASCADE",
            "DROP TABLE IF EXISTS market_ticks CASCADE",
            "DROP TABLE IF EXISTS instruments CASCADE",
            "DROP TABLE IF EXISTS _sqlx_migrations CASCADE",
        ];

        for query in drop_queries {
            sqlx::query(query).execute(&self.pool).await?;
        }

        info!("Database reset completed");
        Ok(())
    }

    /// Count migration files in the migrations directory
    fn count_migration_files(&self) -> usize {
        // This is a simple implementation - in production you might want to
        // actually read the migrations directory
        6 // We have 6 migration files
    }
}

/// Migration status information
#[derive(Debug)]
pub struct MigrationStatus {
    pub applied_migrations: usize,
    pub pending_migrations: usize,
    pub last_migration: Option<String>,
    pub database_ready: bool,
}

/// Schema validation result
#[derive(Debug)]
pub struct SchemaValidation {
    pub valid: bool,
    pub issues: Vec<String>,
}

impl std::fmt::Display for MigrationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Migration Status:\n  Applied: {}\n  Pending: {}\n  Last: {}\n  Ready: {}",
            self.applied_migrations,
            self.pending_migrations,
            self.last_migration.as_deref().unwrap_or("None"),
            self.database_ready
        )
    }
}

impl std::fmt::Display for SchemaValidation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.valid {
            write!(f, "Schema validation: PASSED")
        } else {
            write!(f, "Schema validation: FAILED\nIssues:\n{}", self.issues.join("\n"))
        }
    }
}
