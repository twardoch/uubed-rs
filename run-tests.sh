#!/bin/bash
# this_file: run-tests.sh
# Comprehensive test runner for uubed-rs

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Test results tracking
RUST_TESTS_PASSED=0
PYTHON_TESTS_PASSED=0
C_API_TESTS_PASSED=0
TOTAL_TESTS=0
FAILED_TESTS=0

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -r, --rust        Run Rust tests only"
    echo "  -p, --python      Run Python tests only"
    echo "  -c, --c-api       Run C API tests only"
    echo "  -a, --all         Run all tests (default)"
    echo "  -v, --verbose     Verbose output"
    echo "  -q, --quiet       Quiet output"
    echo "  -f, --fast        Skip slow tests"
    echo "  --coverage        Generate coverage report"
    echo "  --bench           Run benchmarks"
    echo "  -h, --help        Show this help"
    echo ""
    echo "Examples:"
    echo "  $0                # Run all tests"
    echo "  $0 --rust --verbose # Run Rust tests with verbose output"
    echo "  $0 --coverage     # Run tests with coverage"
}

# Run Rust tests
run_rust_tests() {
    local verbose="$1"
    local fast="$2"
    local coverage="$3"
    
    log_info "Running Rust tests..."
    
    cd rust
    
    # Basic unit tests
    log_info "Running unit tests..."
    local cargo_flags="--release"
    if [ "$verbose" = true ]; then
        cargo_flags="$cargo_flags --verbose"
    fi
    
    if [ "$coverage" = true ]; then
        # Install tarpaulin if not present
        if ! command -v cargo-tarpaulin >/dev/null 2>&1; then
            log_info "Installing cargo-tarpaulin..."
            cargo install cargo-tarpaulin
        fi
        
        log_info "Running tests with coverage..."
        cargo tarpaulin --out Html --output-dir ../coverage/rust
        RUST_TESTS_PASSED=1
    else
        if cargo test $cargo_flags --no-default-features --features capi; then
            RUST_TESTS_PASSED=1
            log_success "Rust unit tests passed"
        else
            log_error "Rust unit tests failed"
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi
    fi
    
    # Integration tests
    log_info "Running integration tests..."
    if cargo test $cargo_flags --test integration_test; then
        log_success "Rust integration tests passed"
    else
        log_error "Rust integration tests failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    # Property tests
    if [ "$fast" = false ]; then
        log_info "Running property tests..."
        if cargo test $cargo_flags --test property_tests; then
            log_success "Rust property tests passed"
        else
            log_error "Rust property tests failed"
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi
    fi
    
    # Thread safety tests
    log_info "Running thread safety tests..."
    if cargo test $cargo_flags --test thread_safety; then
        log_success "Rust thread safety tests passed"
    else
        log_error "Rust thread safety tests failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    # Test with all features
    log_info "Running tests with all features..."
    if cargo test $cargo_flags --all-features; then
        log_success "Rust all-features tests passed"
    else
        log_error "Rust all-features tests failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    # Test fuzzing targets (quick run)
    if [ "$fast" = false ] && [ -d "fuzz" ]; then
        log_info "Running fuzz tests (quick)..."
        if command -v cargo-fuzz >/dev/null 2>&1; then
            cd fuzz
            for target in fuzz_targets/*.rs; do
                target_name=$(basename "$target" .rs)
                log_info "Running fuzz target: $target_name"
                if timeout 30 cargo fuzz run "$target_name" -- -max_total_time=30; then
                    log_success "Fuzz target $target_name passed"
                else
                    log_warning "Fuzz target $target_name timed out or failed"
                fi
            done
            cd ..
        else
            log_warning "cargo-fuzz not installed, skipping fuzz tests"
        fi
    fi
    
    cd ..
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

# Run Python tests
run_python_tests() {
    local verbose="$1"
    local coverage="$2"
    
    log_info "Running Python tests..."
    
    if [ ! -d "tests" ]; then
        log_warning "No Python tests directory found"
        return
    fi
    
    # Build Python extension first
    log_info "Building Python extension..."
    if command -v maturin >/dev/null 2>&1; then
        if maturin develop --release --features simd; then
            log_success "Python extension built successfully"
        else
            log_error "Failed to build Python extension"
            FAILED_TESTS=$((FAILED_TESTS + 1))
            return
        fi
    elif command -v uv >/dev/null 2>&1; then
        if uv run maturin develop --release --features simd; then
            log_success "Python extension built successfully"
        else
            log_error "Failed to build Python extension"
            FAILED_TESTS=$((FAILED_TESTS + 1))
            return
        fi
    else
        log_error "Neither maturin nor uv found, cannot build Python extension"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return
    fi
    
    # Run Python tests
    local pytest_flags="-v"
    if [ "$verbose" = true ]; then
        pytest_flags="$pytest_flags -s"
    fi
    
    if [ "$coverage" = true ]; then
        pytest_flags="$pytest_flags --cov=uubed --cov-report=html:coverage/python"
    fi
    
    if command -v uv >/dev/null 2>&1; then
        if uv run python -m pytest tests/ $pytest_flags; then
            PYTHON_TESTS_PASSED=1
            log_success "Python tests passed"
        else
            log_error "Python tests failed"
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi
    else
        if python -m pytest tests/ $pytest_flags; then
            PYTHON_TESTS_PASSED=1
            log_success "Python tests passed"
        else
            log_error "Python tests failed"
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi
    fi
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

# Run C API tests
run_c_api_tests() {
    local verbose="$1"
    
    log_info "Running C API tests..."
    
    # Build C API first
    log_info "Building C API..."
    if make build examples; then
        log_success "C API built successfully"
    else
        log_error "Failed to build C API"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return
    fi
    
    # Run C API demo
    log_info "Running C API demo..."
    if make test-c; then
        C_API_TESTS_PASSED=1
        log_success "C API tests passed"
    else
        log_error "C API tests failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

# Run benchmarks
run_benchmarks() {
    log_info "Running benchmarks..."
    
    cd rust
    
    # Run all benchmarks
    if cargo bench --no-run; then
        log_success "Benchmarks compiled successfully"
        
        # Run specific benchmark suites
        for bench in topk_bench memory_bench large_embedding_bench comparative_bench; do
            log_info "Running benchmark: $bench"
            if cargo bench --bench "$bench"; then
                log_success "Benchmark $bench completed"
            else
                log_warning "Benchmark $bench failed"
            fi
        done
    else
        log_error "Failed to compile benchmarks"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    cd ..
}

# Print test summary
print_summary() {
    echo ""
    echo "================================================"
    echo "                TEST SUMMARY"
    echo "================================================"
    
    local total_suites=$((RUST_TESTS_PASSED + PYTHON_TESTS_PASSED + C_API_TESTS_PASSED))
    local failed_suites=$((TOTAL_TESTS - total_suites))
    
    echo "Test Suites:"
    if [ $RUST_TESTS_PASSED -eq 1 ]; then
        echo -e "  ${GREEN}✓${NC} Rust tests"
    else
        echo -e "  ${RED}✗${NC} Rust tests"
    fi
    
    if [ $PYTHON_TESTS_PASSED -eq 1 ]; then
        echo -e "  ${GREEN}✓${NC} Python tests"
    else
        echo -e "  ${RED}✗${NC} Python tests"
    fi
    
    if [ $C_API_TESTS_PASSED -eq 1 ]; then
        echo -e "  ${GREEN}✓${NC} C API tests"
    else
        echo -e "  ${RED}✗${NC} C API tests"
    fi
    
    echo ""
    echo "Summary: $total_suites/$TOTAL_TESTS test suites passed"
    
    if [ $FAILED_TESTS -gt 0 ]; then
        echo -e "${RED}$FAILED_TESTS test(s) failed${NC}"
        exit 1
    else
        echo -e "${GREEN}All tests passed!${NC}"
    fi
}

# Main execution
main() {
    local run_rust=false
    local run_python=false
    local run_c_api=false
    local run_all=true
    local verbose=false
    local quiet=false
    local fast=false
    local coverage=false
    local bench=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -r|--rust)
                run_rust=true
                run_all=false
                shift
                ;;
            -p|--python)
                run_python=true
                run_all=false
                shift
                ;;
            -c|--c-api)
                run_c_api=true
                run_all=false
                shift
                ;;
            -a|--all)
                run_all=true
                shift
                ;;
            -v|--verbose)
                verbose=true
                shift
                ;;
            -q|--quiet)
                quiet=true
                shift
                ;;
            -f|--fast)
                fast=true
                shift
                ;;
            --coverage)
                coverage=true
                shift
                ;;
            --bench)
                bench=true
                shift
                ;;
            -h|--help)
                print_usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                print_usage
                exit 1
                ;;
        esac
    done
    
    # Create coverage directory if needed
    if [ "$coverage" = true ]; then
        mkdir -p coverage/rust coverage/python
    fi
    
    # Execute requested tests
    if [ "$run_all" = true ]; then
        run_rust_tests "$verbose" "$fast" "$coverage"
        run_python_tests "$verbose" "$coverage"
        run_c_api_tests "$verbose"
    else
        if [ "$run_rust" = true ]; then
            run_rust_tests "$verbose" "$fast" "$coverage"
        fi
        
        if [ "$run_python" = true ]; then
            run_python_tests "$verbose" "$coverage"
        fi
        
        if [ "$run_c_api" = true ]; then
            run_c_api_tests "$verbose"
        fi
    fi
    
    # Run benchmarks if requested
    if [ "$bench" = true ]; then
        run_benchmarks
    fi
    
    # Print summary
    if [ "$quiet" = false ]; then
        print_summary
    fi
}

# Check dependencies
check_dependencies() {
    local missing_deps=()
    
    # Check required tools
    if ! command -v cargo >/dev/null 2>&1; then
        missing_deps+=("cargo (Rust toolchain)")
    fi
    
    if ! command -v make >/dev/null 2>&1; then
        missing_deps+=("make")
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        log_error "Missing required dependencies:"
        for dep in "${missing_deps[@]}"; do
            log_error "  - $dep"
        done
        exit 1
    fi
}

# Initialize
check_dependencies
main "$@"