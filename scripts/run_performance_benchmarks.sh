#!/bin/bash

# PantherSwap Edge Performance Benchmark Runner
# Runs comprehensive performance tests for trading engine optimizations

set -e

echo "🚀 PantherSwap Edge Performance Benchmark Suite"
echo "================================================"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from the pantherswap-edge directory"
    exit 1
fi

# Check if .env file exists
if [ ! -f ".env" ]; then
    echo "❌ Error: .env file not found. Please create one with your database configuration."
    exit 1
fi

echo "📋 Pre-benchmark checks..."

# Build the project in release mode for accurate performance testing
echo "🔨 Building project in release mode..."
cargo build --release --bin trading_engine_benchmark

if [ $? -ne 0 ]; then
    echo "❌ Build failed. Please fix compilation errors."
    exit 1
fi

echo "✅ Build successful"

# Check database connectivity
echo "🔍 Checking database connectivity..."
cargo run --release --bin trading_engine_benchmark -- --check-db-only 2>/dev/null || {
    echo "⚠️  Database check failed, but continuing with benchmark..."
}

echo "🏃 Starting performance benchmarks..."
echo ""

# Run the comprehensive benchmark
echo "📊 Running Trading Engine Performance Benchmark..."
echo "This may take several minutes..."
echo ""

# Set environment variables for optimal performance
export RUST_LOG=info
export TOKIO_WORKER_THREADS=8  # Optimize for performance testing

# Run the benchmark with timing
start_time=$(date +%s)

cargo run --release --bin trading_engine_benchmark

end_time=$(date +%s)
duration=$((end_time - start_time))

echo ""
echo "🎯 Benchmark completed in ${duration} seconds"
echo ""

# Optional: Run additional specific benchmarks
echo "🔧 Additional Performance Tests Available:"
echo "  1. cargo run --release --bin throughput_optimizer"
echo "  2. cargo run --release --bin optimized_throughput_engine"
echo "  3. cargo run --release --bin performance_benchmark"
echo ""

# Performance recommendations
echo "💡 Performance Optimization Tips:"
echo "  - Ensure database is running on SSD storage"
echo "  - Use dedicated database connection pool"
echo "  - Monitor CPU and memory usage during high load"
echo "  - Consider horizontal scaling for production"
echo ""

echo "✅ Performance benchmark suite completed successfully!"
echo ""
echo "📈 Key Metrics to Monitor:"
echo "  - Order execution latency (target: <10ms)"
echo "  - Throughput (target: >2000 TPS)"
echo "  - Memory usage and pool efficiency"
echo "  - Lock contention and queue performance"
echo ""
echo "🔍 For detailed analysis, check the logs above and consider running"
echo "   individual benchmark components for specific optimization areas."
