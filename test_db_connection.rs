use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing TimescaleDB Connection with New Credentials");
    println!("====================================================");
    
    let database_url = "postgres://tsdbadmin:r4izcl278usxyomi@v125e8lovc.onbm4slmfi.tsdb.cloud.timescale.com:32916/tsdb?sslmode=require";
    
    // Test basic connection
    println!("📡 Testing basic connection...");
    let pool = PgPoolOptions::new()
        .min_connections(1)
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await?;
    
    println!("✅ Connection established successfully!");
    
    // Test basic query
    println!("🔍 Testing basic query...");
    let version: String = sqlx::query_scalar("SELECT version()")
        .fetch_one(&pool)
        .await?;
    
    println!("📊 Database version: {}", version);
    
    // Test TimescaleDB extension
    println!("🔍 Testing TimescaleDB extension...");
    let timescale_version: Option<String> = sqlx::query_scalar(
        "SELECT extversion FROM pg_extension WHERE extname = 'timescaledb'"
    )
    .fetch_optional(&pool)
    .await?;
    
    match timescale_version {
        Some(version) => println!("✅ TimescaleDB version: {}", version),
        None => println!("⚠️  TimescaleDB extension not found"),
    }
    
    // Test creating a simple table
    println!("🔍 Testing table creation...");
    sqlx::query("DROP TABLE IF EXISTS test_connection")
        .execute(&pool)
        .await?;
    
    sqlx::query(r#"
        CREATE TABLE test_connection (
            id SERIAL PRIMARY KEY,
            timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            test_data TEXT
        )
    "#)
    .execute(&pool)
    .await?;
    
    println!("✅ Table created successfully!");
    
    // Test inserting data
    println!("🔍 Testing data insertion...");
    sqlx::query("INSERT INTO test_connection (test_data) VALUES ($1)")
        .bind("Test data from PantherSwap Edge")
        .execute(&pool)
        .await?;
    
    println!("✅ Data inserted successfully!");
    
    // Test reading data
    println!("🔍 Testing data retrieval...");
    let test_data: String = sqlx::query_scalar("SELECT test_data FROM test_connection LIMIT 1")
        .fetch_one(&pool)
        .await?;
    
    println!("📊 Retrieved data: {}", test_data);
    
    // Test creating hypertable
    println!("🔍 Testing hypertable creation...");
    match sqlx::query("SELECT create_hypertable('test_connection', 'timestamp', if_not_exists => TRUE)")
        .execute(&pool)
        .await {
        Ok(_) => println!("✅ Hypertable created successfully!"),
        Err(e) => println!("⚠️  Hypertable creation failed: {}", e),
    }
    
    // Clean up
    println!("🧹 Cleaning up test table...");
    sqlx::query("DROP TABLE IF EXISTS test_connection")
        .execute(&pool)
        .await?;
    
    println!("✅ Test completed successfully!");
    println!("🎉 New TimescaleDB credentials are working perfectly!");
    
    pool.close().await;
    Ok(())
}
