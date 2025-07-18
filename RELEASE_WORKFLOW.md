# Release Workflow Documentation

This document describes the complete git-tag-based semversioning and release workflow for uubed-rs.

## Overview

The uubed-rs project now has a comprehensive CI/CD system with:
- Git-tag-based semantic versioning
- Automated multiplatform builds
- Comprehensive test suite
- Local development scripts
- GitHub Actions for CI/CD
- Artifact generation for compiled binaries

## Key Files Created

### Local Scripts

1. **`version-manager.sh`** - Version management utility
   - Show current version
   - Bump version (major/minor/patch/prerelease)
   - Create and push git tags
   - Validate version consistency

2. **`build-and-test-and-release.sh`** - Complete build and release script
   - Build project (Rust, Python, C API)
   - Run tests
   - Create packages
   - Handle releases

3. **`run-tests.sh`** - Comprehensive test runner
   - Run Rust tests
   - Run Python tests
   - Run C API tests
   - Generate coverage reports
   - Run benchmarks

### GitHub Actions

1. **`.github/workflows/ci.yml`** - Continuous Integration
   - Runs on push/PR to main/develop
   - Tests on Ubuntu/Windows/macOS
   - Multiple Rust versions (stable/beta)
   - Python testing across versions
   - Security audits
   - Benchmarks

2. **`.github/workflows/release.yml`** - Release Automation
   - Triggers on git tags (v*)
   - Multiplatform binary builds
   - Python wheel generation
   - C API package creation
   - Automatic PyPI publishing
   - GitHub release creation

### Enhanced Tests

1. **`rust/tests/version_test.rs`** - Version management tests
2. **`tests/test_release_workflow.py`** - Python release workflow tests

## Usage

### Local Development

```bash
# Show current version
./version-manager.sh current

# Show next version options
./version-manager.sh next

# Bump version
./version-manager.sh bump patch

# Create release
./version-manager.sh release minor

# Run full build and test
./build-and-test-and-release.sh --full

# Run specific tests
./run-tests.sh --rust
./run-tests.sh --python
./run-tests.sh --c-api
```

### Release Process

1. **Development**: Work on feature branches
2. **Testing**: Use local scripts to test changes
3. **Version Management**: Use version-manager.sh to bump version
4. **Tagging**: Create git tag with version-manager.sh
5. **Automation**: GitHub Actions handles the rest

### Creating a Release

```bash
# Method 1: Using version-manager.sh (recommended)
./version-manager.sh release minor  # Bumps version and creates tag

# Method 2: Manual
git tag v1.1.0
git push origin v1.1.0
```

### GitHub Actions Workflow

When a tag is pushed:
1. **Create Release** job creates GitHub release
2. **Build Rust Binaries** job builds for all platforms
3. **Build Python Wheels** job creates Python packages
4. **Publish Python** job uploads to PyPI
5. **Build C API Packages** job creates C library packages

## Supported Platforms

### Rust Binaries
- Linux (x86_64, aarch64)
- Windows (x86_64)
- macOS (x86_64, Apple Silicon)

### Python Wheels
- Linux, Windows, macOS
- Python 3.8-3.12

### C API
- Cross-platform static and dynamic libraries

## Configuration

### Environment Variables
- `PYPI_TOKEN`: PyPI authentication token for publishing
- `GITHUB_TOKEN`: Automatically provided by GitHub Actions

### Version Consistency
The system ensures consistency between:
- Git tags (source of truth)
- Cargo.toml workspace version
- Python package version
- C API version

## Testing

### Continuous Integration
- **Unit Tests**: Rust library tests
- **Integration Tests**: Cross-language compatibility
- **Property Tests**: Randomized testing
- **Security Audits**: Vulnerability scanning
- **Performance Tests**: Benchmark regression detection

### Local Testing
- **Fast Tests**: `./run-tests.sh --fast`
- **Full Test Suite**: `./run-tests.sh`
- **Coverage Reports**: `./run-tests.sh --coverage`
- **Benchmarks**: `./run-tests.sh --bench`

## Artifacts

### GitHub Releases
- Rust library binaries (tar.gz/zip)
- Python wheels (.whl)
- C API packages (tar.gz)

### PyPI
- Python wheels for all supported platforms

### Documentation
- Rust documentation generated automatically
- API documentation in releases

## Troubleshooting

### Common Issues

1. **Version Inconsistency**
   ```bash
   ./version-manager.sh validate
   ```

2. **Build Failures**
   ```bash
   ./build-and-test-and-release.sh --clean --build
   ```

3. **Test Failures**
   ```bash
   ./run-tests.sh --verbose
   ```

### Debug Information

Check the following for issues:
- Git repository status
- Cargo.toml version alignment
- Working directory cleanliness
- Required dependencies (jq, cargo, etc.)

## Future Enhancements

Potential improvements to consider:
- Automated changelog generation
- Release notes from git commits
- Docker image generation
- Additional target platforms
- Performance regression alerts
- Dependency update automation

## Security

The workflow includes:
- Dependency vulnerability scanning
- Code security audits
- Secure token handling
- Minimal permission requirements

## Support

For issues with the release workflow:
1. Check this documentation
2. Run validation scripts
3. Check GitHub Actions logs
4. Review git tag and version consistency

---

This workflow provides a robust foundation for managing releases of the uubed-rs project with minimal manual intervention while maintaining high quality and security standards.