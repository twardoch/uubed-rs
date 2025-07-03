# Makefile for uubed-rs C API
#
# This Makefile provides convenient targets for building the library,
# running tests, and creating distribution packages.

# Configuration
CARGO = cargo
CC = gcc
CXX = g++
INSTALL = install
PREFIX = /usr/local
LIBDIR = $(PREFIX)/lib
INCLUDEDIR = $(PREFIX)/include
PKGCONFIGDIR = $(LIBDIR)/pkgconfig

# Build configuration
CARGO_FLAGS = --release
CFLAGS = -Wall -Wextra -std=c99 -O2 -g
CXXFLAGS = -Wall -Wextra -std=c++17 -O2 -g
LDFLAGS = -L./target/release

# Library names
LIB_NAME = libuubed_native
DYLIB_EXT = .so
STATIC_EXT = .a

# Platform-specific settings
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    DYLIB_EXT = .dylib
    LDFLAGS += -Wl,-rpath,@loader_path
endif
ifeq ($(UNAME_S),Linux)
    LDFLAGS += -Wl,-rpath,$$ORIGIN
endif

# Targets
.PHONY: all build test clean install uninstall examples docs help

# Default target
all: build examples

# Build the Rust library for C API
build:
	@echo "Building uubed-rs library..."
	cd rust && $(CARGO) build $(CARGO_FLAGS) --no-default-features --features capi
	@echo "✓ Library built successfully"

# Build the Rust library with Python bindings
build-python:
	@echo "Building uubed-rs library with Python bindings..."
	cd rust && $(CARGO) build $(CARGO_FLAGS) --features python,capi
	@echo "✓ Library with Python bindings built successfully"

# Run Rust tests
test: build
	@echo "Running Rust tests..."
	cd rust && $(CARGO) test $(CARGO_FLAGS) --no-default-features --features capi
	@echo "✓ Rust tests passed"

# Run C API tests (if compiled)
test-c: examples/c_api_demo
	@echo "Running C API demo..."
	LD_LIBRARY_PATH=rust/target/release ./examples/c_api_demo
	@echo "✓ C API demo completed"

# Build examples
examples: examples/c_api_demo

examples/c_api_demo: examples/c_api_demo.c include/uubed.h build
	@echo "Building C API demo..."
	@mkdir -p examples
	$(CC) $(CFLAGS) -Iinclude $< -o $@ $(LDFLAGS) -luubed_native
	@echo "✓ C API demo built"

# Generate documentation
docs:
	@echo "Generating documentation..."
	cd rust && $(CARGO) doc --no-deps $(CARGO_FLAGS)
	@echo "✓ Documentation generated in rust/target/release/doc/"

# Create pkg-config file
uubed.pc: uubed.pc.in
	@echo "Generating pkg-config file..."
	sed -e 's|@PREFIX@|$(PREFIX)|g' \
	    -e 's|@VERSION@|$(shell cd rust && $(CARGO) metadata --no-deps --format-version 1 | jq -r '.packages[0].version')|g' \
	    $< > $@
	@echo "✓ pkg-config file generated"

# Install library and headers
install: build uubed.pc
	@echo "Installing uubed library..."
	$(INSTALL) -d $(DESTDIR)$(LIBDIR)
	$(INSTALL) -d $(DESTDIR)$(INCLUDEDIR)
	$(INSTALL) -d $(DESTDIR)$(PKGCONFIGDIR)
	
	# Install library files
	$(INSTALL) -m 644 rust/target/release/$(LIB_NAME)$(DYLIB_EXT) $(DESTDIR)$(LIBDIR)/
	$(INSTALL) -m 644 rust/target/release/$(LIB_NAME)$(STATIC_EXT) $(DESTDIR)$(LIBDIR)/
	
	# Install header
	$(INSTALL) -m 644 include/uubed.h $(DESTDIR)$(INCLUDEDIR)/
	
	# Install pkg-config file
	$(INSTALL) -m 644 uubed.pc $(DESTDIR)$(PKGCONFIGDIR)/
	
	# Update library cache on Linux
	@if [ "$(UNAME_S)" = "Linux" ] && [ -x /sbin/ldconfig ]; then \
		echo "Updating library cache..."; \
		/sbin/ldconfig; \
	fi
	
	@echo "✓ Installation completed"
	@echo "  Library: $(LIBDIR)/$(LIB_NAME)*"
	@echo "  Header:  $(INCLUDEDIR)/uubed.h"
	@echo "  pkg-config: $(PKGCONFIGDIR)/uubed.pc"

# Uninstall library and headers
uninstall:
	@echo "Uninstalling uubed library..."
	rm -f $(DESTDIR)$(LIBDIR)/$(LIB_NAME)$(DYLIB_EXT)
	rm -f $(DESTDIR)$(LIBDIR)/$(LIB_NAME)$(STATIC_EXT)
	rm -f $(DESTDIR)$(INCLUDEDIR)/uubed.h
	rm -f $(DESTDIR)$(PKGCONFIGDIR)/uubed.pc
	@echo "✓ Uninstallation completed"

# Run benchmarks
bench: build
	@echo "Running benchmarks..."
	cd rust && $(CARGO) bench $(CARGO_FLAGS)

# Run comparative benchmarks
bench-comparative: build
	@echo "Running comparative benchmarks..."
	cd rust && $(CARGO) bench --bench comparative_bench $(CARGO_FLAGS)

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cd rust && $(CARGO) clean
	rm -f examples/c_api_demo
	rm -f uubed.pc
	@echo "✓ Clean completed"

# Development targets
dev-build:
	@echo "Building for development..."
	cd rust && $(CARGO) build

dev-test: dev-build
	@echo "Running development tests..."
	cd rust && $(CARGO) test

# Check code formatting and linting
check:
	@echo "Checking code format and linting..."
	cd rust && $(CARGO) fmt --check
	cd rust && $(CARGO) clippy -- -D warnings
	@echo "✓ Code checks passed"

# Format code
fmt:
	@echo "Formatting code..."
	cd rust && $(CARGO) fmt
	@echo "✓ Code formatted"

# Security audit
audit:
	@echo "Running security audit..."
	cd rust && $(CARGO) audit
	@echo "✓ Security audit completed"

# Create release package
package: build docs
	@echo "Creating release package..."
	@VERSION=$$(cd rust && $(CARGO) metadata --no-deps --format-version 1 | jq -r '.packages[0].version'); \
	PACKAGE_NAME="uubed-rs-$$VERSION"; \
	mkdir -p dist/$$PACKAGE_NAME; \
	cp -r include dist/$$PACKAGE_NAME/; \
	cp rust/target/release/$(LIB_NAME)* dist/$$PACKAGE_NAME/; \
	cp README.md dist/$$PACKAGE_NAME/ 2>/dev/null || echo "README.md not found"; \
	cp examples/c_api_demo.c dist/$$PACKAGE_NAME/; \
	cp Makefile dist/$$PACKAGE_NAME/; \
	cp uubed.pc.in dist/$$PACKAGE_NAME/; \
	cd dist && tar czf $$PACKAGE_NAME.tar.gz $$PACKAGE_NAME; \
	echo "✓ Package created: dist/$$PACKAGE_NAME.tar.gz"

# Help target
help:
	@echo "uubed-rs Makefile"
	@echo "=================="
	@echo ""
	@echo "Build targets:"
	@echo "  all            - Build library and examples (default)"
	@echo "  build          - Build the Rust library"
	@echo "  examples       - Build C API examples"
	@echo "  docs           - Generate documentation"
	@echo ""
	@echo "Test targets:"
	@echo "  test           - Run Rust tests"
	@echo "  test-c         - Run C API demo"
	@echo "  bench          - Run performance benchmarks"
	@echo "  bench-comparative - Run comparative benchmarks"
	@echo ""
	@echo "Installation targets:"
	@echo "  install        - Install library and headers (requires sudo)"
	@echo "  uninstall      - Remove installed files (requires sudo)"
	@echo "  uubed.pc       - Generate pkg-config file"
	@echo ""
	@echo "Development targets:"
	@echo "  dev-build      - Build for development (debug mode)"
	@echo "  dev-test       - Run development tests"
	@echo "  check          - Check code format and linting"
	@echo "  fmt            - Format code"
	@echo "  audit          - Run security audit"
	@echo ""
	@echo "Utility targets:"
	@echo "  clean          - Clean build artifacts"
	@echo "  package        - Create release package"
	@echo "  help           - Show this help message"
	@echo ""
	@echo "Configuration:"
	@echo "  PREFIX=$(PREFIX)"
	@echo "  CC=$(CC)"
	@echo "  CARGO=$(CARGO)"

# Version information
version:
	@cd rust && $(CARGO) metadata --no-deps --format-version 1 | jq -r '.packages[0].version'