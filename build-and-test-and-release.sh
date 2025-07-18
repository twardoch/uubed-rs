#!/bin/bash
# this_file: build-and-test-and-release.sh
# Local build, test, and release script for uubed-rs

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
    echo "  -b, --build       Build the project"
    echo "  -t, --test        Run tests"
    echo "  -r, --release     Create a release (requires git tag)"
    echo "  -c, --clean       Clean build artifacts"
    echo "  -f, --full        Run full pipeline: clean, build, test"
    echo "  -p, --package     Create distribution packages"
    echo "  -h, --help        Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 --full         # Run complete build and test pipeline"
    echo "  $0 --build --test # Build and test only"
    echo "  $0 --release      # Create release from current git tag"
}

# Get current version from git tag or fallback to Cargo.toml
get_version() {
    local version
    if git describe --tags --exact-match HEAD 2>/dev/null; then
        version=$(git describe --tags --exact-match HEAD 2>/dev/null | sed 's/^v//')
    else
        version=$(cd rust && cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
        log_warning "No git tag found for current commit, using Cargo.toml version: $version"
    fi
    echo "$version"
}

# Update version in Cargo.toml files
update_version() {
    local version="$1"
    log_info "Updating version to $version"
    
    # Update workspace version
    sed -i.bak "s/^version = \".*\"/version = \"$version\"/" Cargo.toml
    
    # Update rust package version if it exists
    if [ -f "rust/Cargo.toml" ]; then
        # Check if version is set to workspace
        if grep -q "version.workspace = true" rust/Cargo.toml; then
            log_info "Rust package uses workspace version"
        else
            sed -i.bak "s/^version = \".*\"/version = \"$version\"/" rust/Cargo.toml
        fi
    fi
    
    # Clean up backup files
    find . -name "*.bak" -delete
    
    log_success "Version updated to $version"
}

# Clean build artifacts
clean_build() {
    log_info "Cleaning build artifacts..."
    
    # Clean Rust artifacts
    cd rust && cargo clean
    cd ..
    
    # Clean Python artifacts
    rm -rf dist/
    rm -rf build/
    rm -rf target/
    find . -name "*.pyc" -delete
    find . -name "__pycache__" -type d -exec rm -rf {} + 2>/dev/null || true
    
    # Clean C artifacts
    rm -f examples/c_api_demo
    rm -f uubed.pc
    
    log_success "Build artifacts cleaned"
}

# Build the project
build_project() {
    log_info "Building project..."
    
    # Build Rust library
    log_info "Building Rust library..."
    cd rust && cargo build --release --no-default-features --features capi
    cd ..
    
    # Build with Python bindings
    log_info "Building Python bindings..."
    cd rust && cargo build --release --features python,capi
    cd ..
    
    # Build C API demo
    log_info "Building C API demo..."
    make examples
    
    log_success "Project built successfully"
}

# Run tests
run_tests() {
    log_info "Running tests..."
    
    # Run Rust tests
    log_info "Running Rust tests..."
    cd rust && cargo test --release --no-default-features --features capi
    cd ..
    
    # Run C API tests
    log_info "Running C API tests..."
    make test-c
    
    # Run Python tests if available
    if [ -d "tests" ] && [ -f "pyproject.toml" ]; then
        log_info "Running Python tests..."
        if command -v uv >/dev/null 2>&1; then
            uv run python -m pytest tests/ -v
        else
            python -m pytest tests/ -v
        fi
    fi
    
    log_success "All tests passed"
}

# Create distribution packages
create_packages() {
    log_info "Creating distribution packages..."
    
    local version
    version=$(get_version)
    
    # Create C library package
    log_info "Creating C library package..."
    make package
    
    # Create Python wheel if maturin is available
    if command -v maturin >/dev/null 2>&1; then
        log_info "Creating Python wheel..."
        maturin build --release --features simd
    elif command -v uv >/dev/null 2>&1; then
        log_info "Creating Python wheel with uv..."
        uv build --wheel
    else
        log_warning "maturin or uv not found, skipping Python wheel creation"
    fi
    
    log_success "Distribution packages created"
}

# Create release
create_release() {
    log_info "Creating release..."
    
    # Check if we're on a git tag
    if ! git describe --tags --exact-match HEAD >/dev/null 2>&1; then
        log_error "Not on a git tag. Please create and push a tag first:"
        log_error "  git tag v1.0.0"
        log_error "  git push origin v1.0.0"
        exit 1
    fi
    
    local version
    version=$(get_version)
    log_info "Creating release for version $version"
    
    # Update version in files
    update_version "$version"
    
    # Build and test
    build_project
    run_tests
    
    # Create packages
    create_packages
    
    log_success "Release $version created successfully"
    log_info "Release artifacts are in the dist/ directory"
}

# Run linting and formatting
run_linting() {
    log_info "Running linting and formatting..."
    
    cd rust
    
    # Format code
    cargo fmt
    
    # Run clippy
    cargo clippy --all-features -- -D warnings
    
    cd ..
    
    log_success "Linting and formatting completed"
}

# Run benchmarks
run_benchmarks() {
    log_info "Running benchmarks..."
    
    cd rust && cargo bench --release
    cd ..
    
    log_success "Benchmarks completed"
}

# Main execution
main() {
    local build=false
    local test=false
    local release=false
    local clean=false
    local full=false
    local package=false
    local lint=false
    local bench=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -b|--build)
                build=true
                shift
                ;;
            -t|--test)
                test=true
                shift
                ;;
            -r|--release)
                release=true
                shift
                ;;
            -c|--clean)
                clean=true
                shift
                ;;
            -f|--full)
                full=true
                shift
                ;;
            -p|--package)
                package=true
                shift
                ;;
            -l|--lint)
                lint=true
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
    
    # If no arguments, show help
    if [ $# -eq 0 ] && [ "$build" = false ] && [ "$test" = false ] && [ "$release" = false ] && [ "$clean" = false ] && [ "$full" = false ] && [ "$package" = false ] && [ "$lint" = false ] && [ "$bench" = false ]; then
        print_usage
        exit 0
    fi
    
    # Execute requested operations
    if [ "$clean" = true ]; then
        clean_build
    fi
    
    if [ "$full" = true ]; then
        clean_build
        build_project
        run_tests
    fi
    
    if [ "$build" = true ]; then
        build_project
    fi
    
    if [ "$test" = true ]; then
        run_tests
    fi
    
    if [ "$lint" = true ]; then
        run_linting
    fi
    
    if [ "$bench" = true ]; then
        run_benchmarks
    fi
    
    if [ "$package" = true ]; then
        create_packages
    fi
    
    if [ "$release" = true ]; then
        create_release
    fi
    
    log_success "Script completed successfully"
}

# Check dependencies
check_dependencies() {
    local missing_deps=()
    
    # Check required tools
    if ! command -v cargo >/dev/null 2>&1; then
        missing_deps+=("cargo (Rust toolchain)")
    fi
    
    if ! command -v git >/dev/null 2>&1; then
        missing_deps+=("git")
    fi
    
    if ! command -v jq >/dev/null 2>&1; then
        missing_deps+=("jq")
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