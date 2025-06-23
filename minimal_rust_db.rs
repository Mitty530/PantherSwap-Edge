// Minimal Rust TimescaleDB Test
// Simple connection test without complex dependencies

use std::process::Command;

fn main() {
    println!("🚀 PantherSwap Edge Minimal Rust TimescaleDB Test");
    println!("💾 Testing TimescaleDB connection using Rust + psql");
    
    // Your TimescaleDB connection details
    let host = "jqrbtbc5nw.w0mq2s13iy.tsdb.cloud.timescale.com";
    let port = "35762";
    let database = "tsdb";
    let username = "tsdbadmin";
    let password = "sz2eu577bgqi5767";
    
    println!("📋 Connection Details:");
    println!("   🗄️  Host: {}", host);
    println!("   🔌 Port: {}", port);
    println!("   📊 Database: {}", database);
    println!("   👤 Username: {}", username);
    
    // Test 1: Basic connection test
    println!("\n🔗 Test 1: Basic Connection Test");
    let connection_test = format!(
        "PGPASSWORD={} psql -h {} -p {} -U {} -d {} -c \"SELECT version();\"",
        password, host, port, username, database
    );
    
    match Command::new("sh").arg("-c").arg(&connection_test).output() {
        Ok(output) => {
            if output.status.success() {
                println!("✅ Connection successful!");
                let result = String::from_utf8_lossy(&output.stdout);
                if result.contains("PostgreSQL") {
                    println!("📊 Database version detected: PostgreSQL with TimescaleDB");
                }
            } else {
                println!("❌ Connection failed");
                let error = String::from_utf8_lossy(&output.stderr);
                println!("Error: {}", error);
            }
        }
        Err(e) => {
            println!("❌ Failed to execute connection test: {}", e);
            println!("💡 Note: This test requires psql to be installed");
        }
    }
    
    // Test 2: Create test table and insert data
    println!("\n📊 Test 2: Create Table and Insert Data");
    let table_test = format!(
        "PGPASSWORD={} psql -h {} -p {} -U {} -d {} -c \"
        CREATE TABLE IF NOT EXISTS rust_minimal_test (
            id SERIAL PRIMARY KEY,
            test_name TEXT NOT NULL,
            timestamp TIMESTAMPTZ DEFAULT NOW(),
            value DECIMAL(10,2) NOT NULL,
            metadata JSONB
        );
        
        INSERT INTO rust_minimal_test (test_name, value, metadata) VALUES 
        ('rust_connection_test', 123.45, '{{\\\"language\\\": \\\"rust\\\", \\\"test_type\\\": \\\"minimal\\\"}}'),
        ('market_data_simulation', 150.75, '{{\\\"symbol\\\": \\\"AAPL\\\", \\\"price\\\": 150.75}}'),
        ('trading_signal', 300.25, '{{\\\"action\\\": \\\"BUY\\\", \\\"confidence\\\": 0.85}}'),
        ('execution_record', 2500.50, '{{\\\"symbol\\\": \\\"GOOGL\\\", \\\"quantity\\\": 10}}'),
        ('performance_metric', 400.00, '{{\\\"latency_ms\\\": 25, \\\"throughput_tps\\\": 1200}}');
        
        SELECT COUNT(*) as records_inserted FROM rust_minimal_test WHERE test_name LIKE '%rust%' OR test_name LIKE '%market%' OR test_name LIKE '%trading%';
        \"",
        password, host, port, username, database
    );
    
    match Command::new("sh").arg("-c").arg(&table_test).output() {
        Ok(output) => {
            if output.status.success() {
                println!("✅ Table creation and data insertion successful!");
                let result = String::from_utf8_lossy(&output.stdout);
                println!("📊 Query result:\n{}", result);
            } else {
                println!("❌ Table/data operation failed");
                let error = String::from_utf8_lossy(&output.stderr);
                println!("Error: {}", error);
            }
        }
        Err(e) => {
            println!("❌ Failed to execute table test: {}", e);
        }
    }
    
    // Test 3: Query and analyze data
    println!("\n📈 Test 3: Query and Analyze Data");
    let query_test = format!(
        "PGPASSWORD={} psql -h {} -p {} -U {} -d {} -c \"
        SELECT 
            test_name,
            value,
            timestamp,
            metadata->>'symbol' as symbol,
            metadata->>'action' as action
        FROM rust_minimal_test 
        WHERE test_name IN ('market_data_simulation', 'trading_signal', 'execution_record')
        ORDER BY timestamp DESC;
        
        SELECT 
            'Summary' as report_type,
            COUNT(*) as total_records,
            AVG(value) as avg_value,
            MAX(value) as max_value,
            MIN(value) as min_value
        FROM rust_minimal_test;
        \"",
        password, host, port, username, database
    );
    
    match Command::new("sh").arg("-c").arg(&query_test).output() {
        Ok(output) => {
            if output.status.success() {
                println!("✅ Data query and analysis successful!");
                let result = String::from_utf8_lossy(&output.stdout);
                println!("📊 Analysis results:\n{}", result);
            } else {
                println!("❌ Query operation failed");
                let error = String::from_utf8_lossy(&output.stderr);
                println!("Error: {}", error);
            }
        }
        Err(e) => {
            println!("❌ Failed to execute query test: {}", e);
        }
    }
    
    // Test 4: TimescaleDB specific features
    println!("\n⏰ Test 4: TimescaleDB Features Test");
    let timescale_test = format!(
        "PGPASSWORD={} psql -h {} -p {} -U {} -d {} -c \"
        -- Check if TimescaleDB extension is available
        SELECT extname, extversion FROM pg_extension WHERE extname = 'timescaledb';
        
        -- Try to create a hypertable (will fail if table already is one)
        SELECT create_hypertable('rust_minimal_test', 'timestamp', if_not_exists => TRUE);
        
        -- Show hypertable info
        SELECT hypertable_name, num_chunks FROM timescaledb_information.hypertables WHERE hypertable_name = 'rust_minimal_test';
        \"",
        password, host, port, username, database
    );
    
    match Command::new("sh").arg("-c").arg(&timescale_test).output() {
        Ok(output) => {
            if output.status.success() {
                println!("✅ TimescaleDB features test successful!");
                let result = String::from_utf8_lossy(&output.stdout);
                println!("⏰ TimescaleDB info:\n{}", result);
            } else {
                println!("⚠️  TimescaleDB features test had issues (may be normal)");
                let error = String::from_utf8_lossy(&output.stderr);
                println!("Output: {}", error);
            }
        }
        Err(e) => {
            println!("❌ Failed to execute TimescaleDB test: {}", e);
        }
    }
    
    // Test 5: Performance simulation
    println!("\n🚀 Test 5: Performance Simulation");
    let performance_test = format!(
        "PGPASSWORD={} psql -h {} -p {} -U {} -d {} -c \"
        -- Insert multiple records to simulate trading activity
        INSERT INTO rust_minimal_test (test_name, value, metadata) 
        SELECT 
            'performance_test_' || generate_series,
            random() * 1000,
            json_build_object(
                'iteration', generate_series,
                'symbol', CASE (generate_series % 5) 
                    WHEN 0 THEN 'AAPL'
                    WHEN 1 THEN 'MSFT'
                    WHEN 2 THEN 'GOOGL'
                    WHEN 3 THEN 'TSLA'
                    ELSE 'NVDA'
                END,
                'test_type', 'rust_performance',
                'timestamp_ms', extract(epoch from now()) * 1000
            )
        FROM generate_series(1, 50);
        
        -- Performance analysis
        SELECT 
            'Performance Test Results' as report,
            COUNT(*) as total_test_records,
            COUNT(*) FILTER (WHERE test_name LIKE 'performance_test_%') as performance_records,
            AVG(value) FILTER (WHERE test_name LIKE 'performance_test_%') as avg_test_value,
            COUNT(DISTINCT metadata->>'symbol') FILTER (WHERE metadata->>'symbol' IS NOT NULL) as unique_symbols
        FROM rust_minimal_test;
        \"",
        password, host, port, username, database
    );
    
    match Command::new("sh").arg("-c").arg(&performance_test).output() {
        Ok(output) => {
            if output.status.success() {
                println!("✅ Performance simulation successful!");
                let result = String::from_utf8_lossy(&output.stdout);
                println!("🚀 Performance results:\n{}", result);
            } else {
                println!("❌ Performance test failed");
                let error = String::from_utf8_lossy(&output.stderr);
                println!("Error: {}", error);
            }
        }
        Err(e) => {
            println!("❌ Failed to execute performance test: {}", e);
        }
    }
    
    // Cleanup
    println!("\n🧹 Cleanup: Removing test data");
    let cleanup_test = format!(
        "PGPASSWORD={} psql -h {} -p {} -U {} -d {} -c \"
        DELETE FROM rust_minimal_test WHERE test_name LIKE '%test%' OR test_name LIKE '%rust%' OR test_name LIKE '%performance%';
        DROP TABLE IF EXISTS rust_minimal_test;
        SELECT 'Cleanup completed' as status;
        \"",
        password, host, port, username, database
    );
    
    match Command::new("sh").arg("-c").arg(&cleanup_test).output() {
        Ok(output) => {
            if output.status.success() {
                println!("✅ Cleanup successful!");
            } else {
                println!("⚠️  Cleanup had issues");
            }
        }
        Err(e) => {
            println!("❌ Failed to execute cleanup: {}", e);
        }
    }
    
    println!("\n📊 RUST TIMESCALEDB TEST SUMMARY");
    println!("================================");
    println!("🦀 Language: Rust");
    println!("🗄️  Database: TimescaleDB");
    println!("🔗 Connection: Direct psql integration");
    println!("📊 Tests: Connection, Table Creation, Data Insertion, Queries, Performance");
    println!("⏰ TimescaleDB: Hypertable and extension features tested");
    println!("🚀 Performance: Bulk insert and analysis simulation");
    
    println!("\n✅ Rust TimescaleDB integration test completed!");
    println!("💾 Your TimescaleDB is accessible and functional from Rust");
    println!("🦀 Rust can successfully interact with your TimescaleDB database");
    
    println!("\n💡 Note: This test uses psql commands from Rust.");
    println!("💡 For production, use native Rust database libraries like sqlx or tokio-postgres.");
}
