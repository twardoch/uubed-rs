# Manual Setup Guide for GitHub Actions

Due to GitHub App permissions, the workflow files need to be added manually. Here's how to complete the setup:

## 1. Create GitHub Actions Workflows

You need to manually create these two files in your repository:

### `.github/workflows/ci.yml`
This file handles continuous integration testing.

### `.github/workflows/release.yml`
This file handles automated releases when you push git tags.

## 2. Workflow File Contents

### Create `.github/workflows/ci.yml`:
```yaml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test Suite
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta]
        exclude:
          - os: windows-latest
            rust: beta
          - os: macos-latest
            rust: beta
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        toolchain: ${{ matrix.rust }}
        components: rustfmt, clippy
    
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
      with:
        workspaces: rust
    
    - name: Install system dependencies (Linux)
      if: matrix.os == 'ubuntu-latest'
      run: |
        sudo apt-get update
        sudo apt-get install -y build-essential jq
    
    - name: Install system dependencies (macOS)
      if: matrix.os == 'macos-latest'
      run: |
        brew install jq
    
    - name: Install system dependencies (Windows)
      if: matrix.os == 'windows-latest'
      run: |
        choco install jq
        
    - name: Check formatting
      run: cd rust && cargo fmt --check
      
    - name: Run clippy
      run: cd rust && cargo clippy --all-features -- -D warnings
      
    - name: Run tests
      run: cd rust && cargo test --release --no-default-features --features capi
      
    - name: Run tests with all features
      run: cd rust && cargo test --release --all-features
      
    - name: Build C API demo (Unix)
      if: matrix.os != 'windows-latest'
      run: |
        cd rust && cargo build --release --no-default-features --features capi
        cd ..
        make examples
        
    - name: Build C API demo (Windows)
      if: matrix.os == 'windows-latest'
      run: |
        cd rust && cargo build --release --no-default-features --features capi
        cd ..
        gcc -Iinclude examples/c_api_demo.c -o examples/c_api_demo.exe -Lrust/target/release -luubed_native
      
    - name: Test C API demo (Unix)
      if: matrix.os != 'windows-latest'
      run: |
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
          LD_LIBRARY_PATH=rust/target/release ./examples/c_api_demo
        elif [[ "$OSTYPE" == "darwin"* ]]; then
          DYLD_LIBRARY_PATH=rust/target/release ./examples/c_api_demo
        fi
        
    - name: Test C API demo (Windows)
      if: matrix.os == 'windows-latest'
      run: |
        $env:PATH = "rust/target/release;$env:PATH"
        ./examples/c_api_demo.exe

  python-test:
    name: Python Tests
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        python-version: ["3.8", "3.9", "3.10", "3.11", "3.12"]
        exclude:
          - os: windows-latest
            python-version: "3.8"
          - os: windows-latest
            python-version: "3.9"
          - os: macos-latest
            python-version: "3.8"
          - os: macos-latest
            python-version: "3.9"
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        toolchain: stable
    
    - name: Cache Rust dependencies
      uses: Swatinem/rust-cache@v2
      with:
        workspaces: rust
    
    - name: Set up Python ${{ matrix.python-version }}
      uses: actions/setup-python@v5
      with:
        python-version: ${{ matrix.python-version }}
    
    - name: Install Python dependencies
      run: |
        python -m pip install --upgrade pip
        pip install maturin pytest numpy
    
    - name: Build Python package
      run: |
        maturin build --release --features simd
        pip install --find-links dist --force-reinstall uubed-rs
    
    - name: Run Python tests
      run: |
        if [ -d "tests" ]; then
          python -m pytest tests/ -v
        else
          echo "No Python tests found"
        fi
      shell: bash

  security-audit:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        toolchain: stable
    
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
      with:
        workspaces: rust
    
    - name: Install cargo-audit
      run: cargo install cargo-audit
    
    - name: Run security audit
      run: cd rust && cargo audit

  benchmarks:
    name: Benchmarks
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        toolchain: stable
    
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
      with:
        workspaces: rust
    
    - name: Run benchmarks
      run: cd rust && cargo bench --no-run
    
    - name: Store benchmark results
      uses: benchmark-action/github-action-benchmark@v1
      if: github.ref == 'refs/heads/main'
      with:
        name: Rust Benchmark
        tool: 'cargo'
        output-file-path: rust/target/criterion/report/index.html
        github-token: ${{ secrets.GITHUB_TOKEN }}
        auto-push: true
        comment-on-alert: true
        alert-threshold: '200%'
        fail-on-alert: true
```

### Create `.github/workflows/release.yml`:
```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

env:
  CARGO_TERM_COLOR: always

jobs:
  create-release:
    name: Create Release
    runs-on: ubuntu-latest
    outputs:
      upload_url: ${{ steps.create_release.outputs.upload_url }}
      version: ${{ steps.get_version.outputs.version }}
    steps:
    - uses: actions/checkout@v4
    
    - name: Get version from tag
      id: get_version
      run: |
        VERSION=${GITHUB_REF#refs/tags/v}
        echo "version=$VERSION" >> $GITHUB_OUTPUT
        echo "Version: $VERSION"
    
    - name: Create Release
      id: create_release
      uses: actions/create-release@v1
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      with:
        tag_name: ${{ github.ref }}
        release_name: Release ${{ github.ref }}
        body: |
          Changes in this Release
          
          - See [CHANGELOG.md](https://github.com/twardoch/uubed-rs/blob/main/CHANGELOG.md) for details
          
          ## Installation
          
          ### Rust Library
          Download the appropriate library for your platform from the assets below.
          
          ### Python Package
          ```bash
          pip install uubed-rs
          ```
          
          ### C API
          Download the C API package and follow the installation instructions.
        draft: false
        prerelease: false

  build-rust-binaries:
    name: Build Rust Binaries
    needs: create-release
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            name: linux-x86_64
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            name: linux-aarch64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            name: windows-x86_64
          - os: macos-latest
            target: x86_64-apple-darwin
            name: macos-x86_64
          - os: macos-latest
            target: aarch64-apple-darwin
            name: macos-aarch64
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        toolchain: stable
        targets: ${{ matrix.target }}
    
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
      with:
        workspaces: rust
        key: ${{ matrix.target }}
    
    - name: Install cross-compilation tools (Linux)
      if: matrix.os == 'ubuntu-latest' && matrix.target == 'aarch64-unknown-linux-gnu'
      run: |
        sudo apt-get update
        sudo apt-get install -y gcc-aarch64-linux-gnu
    
    - name: Build binary
      run: |
        cd rust
        cargo build --release --target ${{ matrix.target }} --no-default-features --features capi
    
    - name: Create package
      run: |
        VERSION=${{ needs.create-release.outputs.version }}
        PACKAGE_NAME="uubed-rs-$VERSION-${{ matrix.name }}"
        
        mkdir -p "dist/$PACKAGE_NAME"
        
        # Copy library files
        if [[ "${{ matrix.os }}" == "windows-latest" ]]; then
          cp "rust/target/${{ matrix.target }}/release/uubed_native.dll" "dist/$PACKAGE_NAME/"
          cp "rust/target/${{ matrix.target }}/release/uubed_native.lib" "dist/$PACKAGE_NAME/"
          cp "rust/target/${{ matrix.target }}/release/uubed_native.dll.lib" "dist/$PACKAGE_NAME/" || true
        elif [[ "${{ matrix.os }}" == "macos-latest" ]]; then
          cp "rust/target/${{ matrix.target }}/release/libuubed_native.dylib" "dist/$PACKAGE_NAME/"
          cp "rust/target/${{ matrix.target }}/release/libuubed_native.a" "dist/$PACKAGE_NAME/"
        else
          cp "rust/target/${{ matrix.target }}/release/libuubed_native.so" "dist/$PACKAGE_NAME/"
          cp "rust/target/${{ matrix.target }}/release/libuubed_native.a" "dist/$PACKAGE_NAME/"
        fi
        
        # Copy header and documentation
        cp include/uubed.h "dist/$PACKAGE_NAME/"
        cp README.md "dist/$PACKAGE_NAME/" || echo "README.md not found"
        cp examples/c_api_demo.c "dist/$PACKAGE_NAME/"
        
        # Create archive
        cd dist
        if [[ "${{ matrix.os }}" == "windows-latest" ]]; then
          7z a "$PACKAGE_NAME.zip" "$PACKAGE_NAME"
        else
          tar czf "$PACKAGE_NAME.tar.gz" "$PACKAGE_NAME"
        fi
      shell: bash
    
    - name: Upload Release Asset (tar.gz)
      if: matrix.os != 'windows-latest'
      uses: actions/upload-release-asset@v1
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      with:
        upload_url: ${{ needs.create-release.outputs.upload_url }}
        asset_path: ./dist/uubed-rs-${{ needs.create-release.outputs.version }}-${{ matrix.name }}.tar.gz
        asset_name: uubed-rs-${{ needs.create-release.outputs.version }}-${{ matrix.name }}.tar.gz
        asset_content_type: application/gzip
    
    - name: Upload Release Asset (zip)
      if: matrix.os == 'windows-latest'
      uses: actions/upload-release-asset@v1
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      with:
        upload_url: ${{ needs.create-release.outputs.upload_url }}
        asset_path: ./dist/uubed-rs-${{ needs.create-release.outputs.version }}-${{ matrix.name }}.zip
        asset_name: uubed-rs-${{ needs.create-release.outputs.version }}-${{ matrix.name }}.zip
        asset_content_type: application/zip

  build-python-wheels:
    name: Build Python Wheels
    needs: create-release
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        toolchain: stable
    
    - name: Cache Rust dependencies
      uses: Swatinem/rust-cache@v2
      with:
        workspaces: rust
    
    - name: Set up Python
      uses: actions/setup-python@v5
      with:
        python-version: '3.11'
    
    - name: Install build dependencies
      run: |
        python -m pip install --upgrade pip
        pip install maturin
    
    - name: Update version in Cargo.toml
      run: |
        VERSION=${{ needs.create-release.outputs.version }}
        sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
        rm -f Cargo.toml.bak
      shell: bash
    
    - name: Build wheels
      run: |
        maturin build --release --features simd --strip
    
    - name: Install wheel and test
      run: |
        pip install --find-links dist --force-reinstall uubed-rs
        python -c "import uubed; print('uubed-rs import successful')"
    
    - name: Upload wheels
      uses: actions/upload-artifact@v4
      with:
        name: wheels-${{ matrix.os }}
        path: dist/*.whl

  publish-python:
    name: Publish Python Package
    needs: [create-release, build-python-wheels]
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Set up Python
      uses: actions/setup-python@v5
      with:
        python-version: '3.11'
    
    - name: Download all wheels
      uses: actions/download-artifact@v4
      with:
        pattern: wheels-*
        merge-multiple: true
        path: dist/
    
    - name: Install twine
      run: pip install twine
    
    - name: Publish to PyPI
      env:
        TWINE_USERNAME: __token__
        TWINE_PASSWORD: ${{ secrets.PYPI_TOKEN }}
      run: |
        twine upload dist/*.whl
      if: env.TWINE_PASSWORD != ''
    
    - name: Upload wheels to release
      run: |
        for wheel in dist/*.whl; do
          if [ -f "$wheel" ]; then
            echo "Uploading $wheel"
            gh release upload ${{ github.ref_name }} "$wheel"
          fi
        done
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

  build-c-api-packages:
    name: Build C API Packages
    needs: create-release
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            name: linux
          - os: windows-latest
            name: windows
          - os: macos-latest
            name: macos
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        toolchain: stable
    
    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2
      with:
        workspaces: rust
    
    - name: Install system dependencies (Linux)
      if: matrix.os == 'ubuntu-latest'
      run: |
        sudo apt-get update
        sudo apt-get install -y build-essential jq
    
    - name: Install system dependencies (macOS)
      if: matrix.os == 'macos-latest'
      run: |
        brew install jq
    
    - name: Install system dependencies (Windows)
      if: matrix.os == 'windows-latest'
      run: |
        choco install jq
        choco install make
    
    - name: Update version in Cargo.toml
      run: |
        VERSION=${{ needs.create-release.outputs.version }}
        sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
        rm -f Cargo.toml.bak
      shell: bash
    
    - name: Build and package
      run: |
        make clean
        make build
        make package
      shell: bash
    
    - name: Upload C API package
      run: |
        for package in dist/*.tar.gz; do
          if [ -f "$package" ]; then
            echo "Uploading $package"
            gh release upload ${{ github.ref_name }} "$package"
          fi
        done
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      shell: bash
```

## 3. Optional: Configure PyPI Token

If you want automatic PyPI publishing, add a `PYPI_TOKEN` secret to your GitHub repository:

1. Go to your repository settings
2. Click on "Secrets and variables" → "Actions"
3. Click "New repository secret"
4. Name: `PYPI_TOKEN`
5. Value: Your PyPI API token

## 4. Test the Setup

Once you've added the workflow files:

1. Push them to your repository
2. The CI workflow will run on pushes to main/develop
3. The release workflow will run when you push a git tag

## 5. Using the System

All the local scripts are ready to use:

```bash
# Show current version
./version-manager.sh current

# Create a new release
./version-manager.sh release patch

# Run tests locally
./run-tests.sh

# Build everything
./build-and-test-and-release.sh --full
```

The system is fully functional - you just need to manually add the workflow files due to GitHub App permissions.