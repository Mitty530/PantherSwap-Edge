# Database Deployment Guide

## Overview

This guide covers the deployment, configuration, and operational procedures for the PantherSwap Edge database infrastructure in production environments.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Environment Setup](#environment-setup)
3. [Database Installation](#database-installation)
4. [Configuration](#configuration)
5. [Migration Procedures](#migration-procedures)
6. [Monitoring Setup](#monitoring-setup)
7. [Backup & Recovery](#backup--recovery)
8. [Performance Tuning](#performance-tuning)
9. [Security Configuration](#security-configuration)
10. [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements

**Minimum Production Requirements**:
- **CPU**: 8 cores (16 threads) Intel Xeon or AMD EPYC
- **Memory**: 32 GB RAM (64 GB recommended)
- **Storage**: 1 TB NVMe SSD (enterprise grade)
- **Network**: 10 Gbps network interface
- **OS**: Ubuntu 22.04 LTS or RHEL 8+

**Recommended Production Requirements**:
- **CPU**: 16 cores (32 threads) Intel Xeon or AMD EPYC
- **Memory**: 128 GB RAM
- **Storage**: 2 TB NVMe SSD RAID 10
- **Network**: 25 Gbps network interface with redundancy
- **OS**: Ubuntu 22.04 LTS

### Software Dependencies

```bash
# PostgreSQL 15+
sudo apt update
sudo apt install postgresql-15 postgresql-contrib-15

# TimescaleDB 2.11+
sudo sh -c "echo 'deb https://packagecloud.io/timescale/timescaledb/ubuntu/ $(lsb_release -c -s) main' > /etc/apt/sources.list.d/timescaledb.list"
wget --quiet -O - https://packagecloud.io/timescale/timescaledb/gpgkey | sudo apt-key add -
sudo apt update
sudo apt install timescaledb-2-postgresql-15

# Additional tools
sudo apt install postgresql-15-pgaudit postgresql-15-pg-stat-statements
```

## Environment Setup

### Development Environment

```bash
# Development database setup
export DATABASE_URL="postgres://dev_user:dev_password@localhost:5432/pantherswap_dev"
export RUST_LOG="debug"
export ENVIRONMENT="development"

# Create development database
sudo -u postgres createdb pantherswap_dev
sudo -u postgres psql -c "CREATE USER dev_user WITH PASSWORD 'dev_password';"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE pantherswap_dev TO dev_user;"
```

### Staging Environment

```bash
# Staging database setup
export DATABASE_URL="postgres://staging_user:staging_password@staging-db:5432/pantherswap_staging"
export RUST_LOG="info"
export ENVIRONMENT="staging"

# SSL configuration
export PGSSLMODE="require"
export PGSSLCERT="/etc/ssl/certs/client-cert.pem"
export PGSSLKEY="/etc/ssl/private/client-key.pem"
export PGSSLROOTCERT="/etc/ssl/certs/ca-cert.pem"
```

### Production Environment

```bash
# Production database setup
export DATABASE_URL="postgres://prod_user:$(cat /etc/secrets/db_password)@prod-db:5432/pantherswap_prod"
export RUST_LOG="warn"
export ENVIRONMENT="production"

# SSL and security configuration
export PGSSLMODE="require"
export PGSSLCERT="/etc/ssl/certs/client-cert.pem"
export PGSSLKEY="/etc/ssl/private/client-key.pem"
export PGSSLROOTCERT="/etc/ssl/certs/ca-cert.pem"
export PGCONNECT_TIMEOUT="10"
export PGCOMMAND_TIMEOUT="30"
```

## Database Installation

### PostgreSQL Configuration

**postgresql.conf** optimizations for trading workloads:

```ini
# Connection settings
max_connections = 200
superuser_reserved_connections = 3

# Memory settings
shared_buffers = 32GB                    # 25% of total RAM
effective_cache_size = 96GB              # 75% of total RAM
work_mem = 256MB                         # For complex queries
maintenance_work_mem = 2GB               # For maintenance operations
wal_buffers = 64MB                       # WAL buffer size

# Checkpoint settings
checkpoint_timeout = 15min               # Checkpoint frequency
checkpoint_completion_target = 0.9       # Spread checkpoints
max_wal_size = 4GB                       # Maximum WAL size
min_wal_size = 1GB                       # Minimum WAL size

# Query planner
random_page_cost = 1.1                   # SSD optimization
effective_io_concurrency = 200           # SSD concurrent I/O
seq_page_cost = 1                        # Sequential scan cost

# Logging
log_destination = 'csvlog'
logging_collector = on
log_directory = '/var/log/postgresql'
log_filename = 'postgresql-%Y-%m-%d_%H%M%S.log'
log_rotation_age = 1d
log_rotation_size = 100MB
log_min_duration_statement = 1000        # Log slow queries (1s+)
log_checkpoints = on
log_connections = on
log_disconnections = on
log_lock_waits = on

# Performance monitoring
shared_preload_libraries = 'timescaledb,pg_stat_statements,pgaudit'
track_activities = on
track_counts = on
track_io_timing = on
track_functions = all

# TimescaleDB settings
timescaledb.max_background_workers = 8
timescaledb.telemetry_level = off
```

**pg_hba.conf** security configuration:

```ini
# TYPE  DATABASE        USER            ADDRESS                 METHOD

# Local connections
local   all             postgres                                peer
local   all             all                                     md5

# IPv4 local connections
host    all             all             127.0.0.1/32            md5

# Production connections (SSL required)
hostssl pantherswap_prod prod_user      10.0.0.0/8              cert
hostssl pantherswap_prod backup_user    10.0.0.0/8              cert

# Staging connections
hostssl pantherswap_staging staging_user 10.0.0.0/8            md5

# Monitoring connections
hostssl all             monitoring_user 10.0.0.0/8             md5

# Deny all other connections
host    all             all             0.0.0.0/0               reject
```

### TimescaleDB Setup

```sql
-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Enable additional extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;
CREATE EXTENSION IF NOT EXISTS pgaudit;

-- Configure TimescaleDB
SELECT timescaledb_pre_restore();

-- Set TimescaleDB configuration
ALTER SYSTEM SET timescaledb.max_background_workers = 8;
ALTER SYSTEM SET timescaledb.telemetry_level = 'off';
SELECT pg_reload_conf();
```

## Configuration

### Application Configuration

**config/production.toml**:
```toml
[database]
url = "postgres://prod_user:password@prod-db:5432/pantherswap_prod"
max_connections = 50
min_connections = 10
acquire_timeout_seconds = 5
idle_timeout_seconds = 300
max_lifetime_seconds = 1800
test_before_acquire = true

[database.ssl]
mode = "require"
cert_file = "/etc/ssl/certs/client-cert.pem"
key_file = "/etc/ssl/private/client-key.pem"
ca_file = "/etc/ssl/certs/ca-cert.pem"

[monitoring]
enable_health_monitoring = true
health_check_interval_seconds = 30
enable_performance_monitoring = true
enable_alerting = true

[logging]
level = "warn"
format = "json"
output = "file"
file_path = "/var/log/pantherswap/database.log"
```

### Connection Pool Configuration

```rust
// Production connection pool configuration
let pool_config = DatabasePoolConfig {
    min_connections: 10,
    max_connections: 50,
    acquire_timeout: Duration::from_secs(5),
    idle_timeout: Some(Duration::from_secs(300)),
    max_lifetime: Some(Duration::from_secs(1800)),
    test_before_acquire: true,
};

// High-frequency trading configuration
let hft_config = DatabasePoolConfig {
    min_connections: 20,
    max_connections: 100,
    acquire_timeout: Duration::from_secs(2),
    idle_timeout: Some(Duration::from_secs(120)),
    max_lifetime: Some(Duration::from_secs(900)),
    test_before_acquire: true,
};
```

## Migration Procedures

### Initial Schema Deployment

```bash
# 1. Create database and user
sudo -u postgres createdb pantherswap_prod
sudo -u postgres psql -c "CREATE USER prod_user WITH PASSWORD 'secure_password';"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE pantherswap_prod TO prod_user;"

# 2. Run initial migrations
cd pantherswap-edge
cargo run --bin migrate -- --database-url $DATABASE_URL up

# 3. Verify schema
cargo run --bin migrate -- --database-url $DATABASE_URL status
```

### Production Migration Process

```bash
#!/bin/bash
# production-migration.sh

set -e

DATABASE_URL="$1"
MIGRATION_VERSION="$2"

echo "Starting production migration to version $MIGRATION_VERSION"

# 1. Create backup
echo "Creating pre-migration backup..."
pg_dump $DATABASE_URL > "backup_pre_migration_$(date +%Y%m%d_%H%M%S).sql"

# 2. Validate current state
echo "Validating current database state..."
cargo run --bin migrate -- --database-url $DATABASE_URL validate

# 3. Run migration in transaction
echo "Running migration..."
cargo run --bin migrate -- --database-url $DATABASE_URL up --target $MIGRATION_VERSION

# 4. Validate post-migration
echo "Validating post-migration state..."
cargo run --bin migrate -- --database-url $DATABASE_URL validate

# 5. Run health check
echo "Running health check..."
cargo run --bin health-check -- --database-url $DATABASE_URL

echo "Migration completed successfully"
```

### Rollback Procedures

```bash
#!/bin/bash
# rollback-migration.sh

set -e

DATABASE_URL="$1"
TARGET_VERSION="$2"

echo "Rolling back to version $TARGET_VERSION"

# 1. Create backup before rollback
echo "Creating pre-rollback backup..."
pg_dump $DATABASE_URL > "backup_pre_rollback_$(date +%Y%m%d_%H%M%S).sql"

# 2. Execute rollback
echo "Executing rollback..."
cargo run --bin migrate -- --database-url $DATABASE_URL down --target $TARGET_VERSION

# 3. Validate state
echo "Validating rollback state..."
cargo run --bin migrate -- --database-url $DATABASE_URL validate

echo "Rollback completed successfully"
```

## Monitoring Setup

### Health Monitoring Configuration

```rust
// Health monitoring setup
let health_config = HealthMonitorConfig {
    check_interval_seconds: 30,
    metrics_retention_hours: 24,
    enable_continuous_monitoring: true,
    enable_alerting: true,
    max_history_size: 2880, // 24 hours at 30-second intervals
};

let alert_config = AlertConfig {
    enable_notifications: true,
    alert_cooldown_minutes: 5,
    max_alerts_per_hour: 20,
    escalation_enabled: true,
    escalation_threshold_minutes: 15,
    alert_retention_hours: 168, // 7 days
};
```

### Prometheus Metrics

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'pantherswap-database'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'
    scrape_interval: 5s
```

### Grafana Dashboard

```json
{
  "dashboard": {
    "title": "PantherSwap Database Metrics",
    "panels": [
      {
        "title": "Connection Pool Utilization",
        "type": "graph",
        "targets": [
          {
            "expr": "database_connection_pool_active / database_connection_pool_max * 100"
          }
        ]
      },
      {
        "title": "Query Response Time",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, database_query_duration_seconds_bucket)"
          }
        ]
      },
      {
        "title": "Health Score",
        "type": "singlestat",
        "targets": [
          {
            "expr": "database_health_score"
          }
        ]
      }
    ]
  }
}
```

## Backup & Recovery

### Automated Backup Strategy

```bash
#!/bin/bash
# backup-database.sh

set -e

DATABASE_URL="$1"
BACKUP_DIR="/var/backups/postgresql"
RETENTION_DAYS=30

# Create backup directory
mkdir -p $BACKUP_DIR

# Generate backup filename
BACKUP_FILE="pantherswap_$(date +%Y%m%d_%H%M%S).sql"
BACKUP_PATH="$BACKUP_DIR/$BACKUP_FILE"

# Create backup
echo "Creating backup: $BACKUP_FILE"
pg_dump $DATABASE_URL | gzip > "$BACKUP_PATH.gz"

# Verify backup
echo "Verifying backup..."
gunzip -t "$BACKUP_PATH.gz"

# Upload to S3 (optional)
if [ ! -z "$S3_BUCKET" ]; then
    echo "Uploading to S3..."
    aws s3 cp "$BACKUP_PATH.gz" "s3://$S3_BUCKET/database-backups/"
fi

# Cleanup old backups
echo "Cleaning up old backups..."
find $BACKUP_DIR -name "pantherswap_*.sql.gz" -mtime +$RETENTION_DAYS -delete

echo "Backup completed: $BACKUP_PATH.gz"
```

### Point-in-Time Recovery

```bash
#!/bin/bash
# point-in-time-recovery.sh

set -e

RECOVERY_TIME="$1"  # Format: 2024-01-15 14:30:00
BACKUP_FILE="$2"
NEW_DATABASE="pantherswap_recovery"

echo "Starting point-in-time recovery to $RECOVERY_TIME"

# 1. Create recovery database
sudo -u postgres createdb $NEW_DATABASE

# 2. Restore from backup
echo "Restoring from backup..."
gunzip -c $BACKUP_FILE | sudo -u postgres psql $NEW_DATABASE

# 3. Apply WAL files up to recovery point
echo "Applying WAL files..."
sudo -u postgres pg_waldump /var/lib/postgresql/15/main/pg_wal/ \
    --start=$(date -d "$RECOVERY_TIME" +%s) \
    | sudo -u postgres psql $NEW_DATABASE

echo "Point-in-time recovery completed"
echo "Recovery database: $NEW_DATABASE"
```

## Performance Tuning

### Query Optimization

```sql
-- Enable query performance tracking
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- Analyze slow queries
SELECT 
    query,
    calls,
    total_time,
    mean_time,
    rows
FROM pg_stat_statements 
WHERE mean_time > 100  -- Queries taking more than 100ms
ORDER BY mean_time DESC 
LIMIT 10;

-- Update table statistics
ANALYZE;

-- Reindex if needed
REINDEX DATABASE pantherswap_prod;
```

### Index Optimization

```sql
-- Find missing indexes
SELECT 
    schemaname,
    tablename,
    seq_scan,
    seq_tup_read,
    idx_scan,
    idx_tup_fetch,
    seq_tup_read / seq_scan AS avg_seq_read
FROM pg_stat_user_tables 
WHERE seq_scan > 0 
ORDER BY seq_tup_read DESC;

-- Find unused indexes
SELECT 
    schemaname,
    tablename,
    indexname,
    idx_scan,
    idx_tup_read,
    idx_tup_fetch
FROM pg_stat_user_indexes 
WHERE idx_scan = 0 
ORDER BY pg_relation_size(indexrelid) DESC;
```

### Connection Pool Tuning

```rust
// Monitor pool performance
let pool_stats = database.pool_stats();
println!("Pool utilization: {:.1}%", 
         (pool_stats.active as f64 / pool_stats.max_size as f64) * 100.0);

// Auto-tune pool size based on load
if pool_stats.active as f64 / pool_stats.max_size as f64 > 0.8 {
    // Consider increasing pool size
    warn!("High pool utilization detected");
}
```

## Security Configuration

### SSL/TLS Setup

```bash
# Generate SSL certificates
openssl req -new -x509 -days 365 -nodes -text \
    -out server.crt -keyout server.key \
    -subj "/CN=pantherswap-db"

# Set permissions
chmod 600 server.key
chown postgres:postgres server.key server.crt

# Configure PostgreSQL
echo "ssl = on" >> /etc/postgresql/15/main/postgresql.conf
echo "ssl_cert_file = '/etc/ssl/certs/server.crt'" >> /etc/postgresql/15/main/postgresql.conf
echo "ssl_key_file = '/etc/ssl/private/server.key'" >> /etc/postgresql/15/main/postgresql.conf
```

### User Management

```sql
-- Create application users with minimal privileges
CREATE ROLE pantherswap_app LOGIN PASSWORD 'secure_password';
CREATE ROLE pantherswap_readonly LOGIN PASSWORD 'readonly_password';
CREATE ROLE pantherswap_backup LOGIN PASSWORD 'backup_password';

-- Grant appropriate permissions
GRANT CONNECT ON DATABASE pantherswap_prod TO pantherswap_app;
GRANT USAGE ON SCHEMA public TO pantherswap_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO pantherswap_app;

-- Read-only access for analytics
GRANT CONNECT ON DATABASE pantherswap_prod TO pantherswap_readonly;
GRANT USAGE ON SCHEMA public TO pantherswap_readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO pantherswap_readonly;

-- Backup user permissions
GRANT CONNECT ON DATABASE pantherswap_prod TO pantherswap_backup;
GRANT USAGE ON SCHEMA public TO pantherswap_backup;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO pantherswap_backup;
```

## Troubleshooting

### Common Issues

**High Connection Count**:
```sql
-- Check active connections
SELECT count(*) FROM pg_stat_activity WHERE state = 'active';

-- Kill long-running queries
SELECT pg_terminate_backend(pid) 
FROM pg_stat_activity 
WHERE state = 'active' 
AND query_start < NOW() - INTERVAL '5 minutes';
```

**Slow Queries**:
```sql
-- Find blocking queries
SELECT 
    blocked_locks.pid AS blocked_pid,
    blocked_activity.usename AS blocked_user,
    blocking_locks.pid AS blocking_pid,
    blocking_activity.usename AS blocking_user,
    blocked_activity.query AS blocked_statement,
    blocking_activity.query AS current_statement_in_blocking_process
FROM pg_catalog.pg_locks blocked_locks
JOIN pg_catalog.pg_stat_activity blocked_activity ON blocked_activity.pid = blocked_locks.pid
JOIN pg_catalog.pg_locks blocking_locks ON blocking_locks.locktype = blocked_locks.locktype
JOIN pg_catalog.pg_stat_activity blocking_activity ON blocking_activity.pid = blocking_locks.pid
WHERE NOT blocked_locks.granted;
```

**Disk Space Issues**:
```bash
# Check database sizes
sudo -u postgres psql -c "
SELECT 
    datname,
    pg_size_pretty(pg_database_size(datname)) as size
FROM pg_database 
ORDER BY pg_database_size(datname) DESC;"

# Check table sizes
sudo -u postgres psql pantherswap_prod -c "
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_tables 
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC 
LIMIT 10;"
```

---

This deployment guide provides comprehensive procedures for setting up, configuring, and maintaining the PantherSwap Edge database infrastructure in production environments.
