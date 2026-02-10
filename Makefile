# trex - tmux session manager
# ISC License

BINARY := trex
VERSION := $(shell grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
PREFIX ?= $(HOME)/.cargo
BINDIR := $(PREFIX)/bin

CARGO := cargo
INSTALL := install
RM := rm -f

# Targets
TARGET_NATIVE := $(shell rustc -vV | grep host | cut -d' ' -f2)
TARGET_MUSL_X86 := x86_64-unknown-linux-musl
TARGET_MUSL_ARM := aarch64-unknown-linux-musl

# Output paths
RELEASE_DIR := target/release
STATIC_X86_DIR := target/$(TARGET_MUSL_X86)/release
STATIC_ARM_DIR := target/$(TARGET_MUSL_ARM)/release
DIST_DIR := dist

.PHONY: all build build-ascii release release-ascii rebuild \
        static static-x86 static-arm \
        install install-static install-user install-static-user \
        install-ascii install-ascii-user \
        uninstall uninstall-user \
        run run-ascii doc \
        test fmt fmt-check lint check pre-release pre-commit \
        dist dist-all clean help

# Default target: show help
all: help

# ─── Build ────────────────────────────────────────────────────────────

# Development build
build:
	$(CARGO) build

# Development build with ascii-art feature
build-ascii:
	$(CARGO) build --features ascii-art

# Optimized release build
release:
	$(CARGO) build --release

# Optimized release build with ascii-art feature
release-ascii:
	$(CARGO) build --release --features ascii-art

# Full rebuild from scratch (clears all cargo cache)
rebuild:
	$(CARGO) clean
	$(RM) -r $(HOME)/.cargo/registry/cache
	$(RM) -r $(HOME)/.cargo/git/checkouts
	$(CARGO) build --release

# Static Linux binary (x86_64)
static: static-x86

static-x86:
	@echo "Building static x86_64 binary..."
	@rustup target add $(TARGET_MUSL_X86) 2>/dev/null || true
	$(CARGO) build --release --target $(TARGET_MUSL_X86)
	@echo "Binary: $(STATIC_X86_DIR)/$(BINARY)"

# Static Linux binary (aarch64) - requires cross or appropriate linker
static-arm:
	@echo "Building static aarch64 binary..."
	@rustup target add $(TARGET_MUSL_ARM) 2>/dev/null || true
	@command -v cross >/dev/null 2>&1 && \
		cross build --release --target $(TARGET_MUSL_ARM) || \
		$(CARGO) build --release --target $(TARGET_MUSL_ARM)
	@echo "Binary: $(STATIC_ARM_DIR)/$(BINARY)"

# ─── Install ──────────────────────────────────────────────────────────

# Install to system (default: ~/.cargo/bin)
install: release
	$(INSTALL) -d $(DESTDIR)$(BINDIR)
	$(INSTALL) -m 755 $(RELEASE_DIR)/$(BINARY) $(DESTDIR)$(BINDIR)/$(BINARY)
	@echo "Installed $(BINARY) to $(DESTDIR)$(BINDIR)"

# Install with ascii-art feature to system
install-ascii: release-ascii
	$(INSTALL) -d $(DESTDIR)$(BINDIR)
	$(INSTALL) -m 755 $(RELEASE_DIR)/$(BINARY) $(DESTDIR)$(BINDIR)/$(BINARY)
	@echo "Installed $(BINARY) (ascii-art) to $(DESTDIR)$(BINDIR)"

# Install static binary
install-static: static-x86
	$(INSTALL) -d $(DESTDIR)$(BINDIR)
	$(INSTALL) -m 755 $(STATIC_X86_DIR)/$(BINARY) $(DESTDIR)$(BINDIR)/$(BINARY)
	@echo "Installed static $(BINARY) to $(DESTDIR)$(BINDIR)"

# Install to ~/.cargo/bin (user install)
install-user:
	$(CARGO) install --path .
	@echo "Installed $(BINARY) to ~/.cargo/bin"

# Install with ascii-art feature to ~/.cargo/bin
install-ascii-user:
	$(CARGO) install --path . --features ascii-art
	@echo "Installed $(BINARY) (ascii-art) to ~/.cargo/bin"

# Install static binary to ~/.cargo/bin
install-static-user: static-x86
	$(INSTALL) -d $(HOME)/.cargo/bin
	$(INSTALL) -m 755 $(STATIC_X86_DIR)/$(BINARY) $(HOME)/.cargo/bin/$(BINARY)
	@echo "Installed static $(BINARY) to ~/.cargo/bin"

# Uninstall from system
uninstall:
	$(RM) $(DESTDIR)$(BINDIR)/$(BINARY)
	@echo "Removed $(BINARY) from $(DESTDIR)$(BINDIR)"

# Uninstall from ~/.cargo/bin
uninstall-user:
	$(CARGO) uninstall $(BINARY) 2>/dev/null || true
	@echo "Removed $(BINARY) from ~/.cargo/bin"

# ─── Run ──────────────────────────────────────────────────────────────

# Run debug build
run:
	$(CARGO) run

# Run with ascii-art feature
run-ascii:
	$(CARGO) run --features ascii-art

# ─── Distribution ─────────────────────────────────────────────────────

# Create distribution archives
dist: static-x86
	@mkdir -p $(DIST_DIR)
	@cp $(STATIC_X86_DIR)/$(BINARY) $(DIST_DIR)/$(BINARY)-$(VERSION)-linux-x86_64
	@cd $(DIST_DIR) && tar -czvf $(BINARY)-$(VERSION)-linux-x86_64.tar.gz $(BINARY)-$(VERSION)-linux-x86_64
	@echo "Created $(DIST_DIR)/$(BINARY)-$(VERSION)-linux-x86_64.tar.gz"

# Create distribution for both architectures
dist-all: static-x86 static-arm
	@mkdir -p $(DIST_DIR)
	@cp $(STATIC_X86_DIR)/$(BINARY) $(DIST_DIR)/$(BINARY)-$(VERSION)-linux-x86_64
	@cp $(STATIC_ARM_DIR)/$(BINARY) $(DIST_DIR)/$(BINARY)-$(VERSION)-linux-aarch64
	@cd $(DIST_DIR) && tar -czvf $(BINARY)-$(VERSION)-linux-x86_64.tar.gz $(BINARY)-$(VERSION)-linux-x86_64
	@cd $(DIST_DIR) && tar -czvf $(BINARY)-$(VERSION)-linux-aarch64.tar.gz $(BINARY)-$(VERSION)-linux-aarch64
	@echo "Created archives in $(DIST_DIR)/"

# ─── Development ──────────────────────────────────────────────────────

# Run tests
test:
	$(CARGO) test

# Format code
fmt:
	$(CARGO) fmt

# Check formatting without modifying
fmt-check:
	$(CARGO) fmt --check

# Lint code
lint:
	$(CARGO) clippy -- -D warnings

# Check without building
check:
	$(CARGO) check

# Generate and open documentation
doc:
	$(CARGO) doc --open

# Run pre-commit on all files
pre-commit:
	python3 -m pre_commit run --all-files

# Full pre-release validation
pre-release:
	@echo "══════════════════════════════════════════════"
	@echo "  Pre-release check: $(BINARY) $(VERSION)"
	@echo "══════════════════════════════════════════════"
	@echo ""
	@echo "── Formatting ──"
	$(CARGO) fmt --check
	@echo ""
	@echo "── Linting ──"
	$(CARGO) clippy -- -D warnings
	@echo ""
	@echo "── Tests ──"
	$(CARGO) test
	@echo ""
	@echo "── Build (default) ──"
	$(CARGO) build --release
	@echo ""
	@echo "── Build (ascii-art) ──"
	$(CARGO) build --release --features ascii-art
	@echo ""
	@echo "══════════════════════════════════════════════"
	@echo "  Pre-release check passed"
	@echo "══════════════════════════════════════════════"

# Clean build artifacts
clean:
	$(CARGO) clean
	$(RM) -r $(DIST_DIR)

# ─── Help ─────────────────────────────────────────────────────────────

help:
	@echo "trex $(VERSION) - tmux session manager"
	@echo ""
	@echo "Build:"
	@echo "  make build             Debug build"
	@echo "  make build-ascii       Debug build with ascii-art feature"
	@echo "  make release           Optimized release build"
	@echo "  make release-ascii     Optimized release build with ascii-art feature"
	@echo "  make rebuild           Clean all cache and rebuild from scratch"
	@echo "  make static            Static x86_64 Linux binary (musl)"
	@echo "  make static-x86        Static x86_64 Linux binary (musl)"
	@echo "  make static-arm        Static aarch64 Linux binary (musl)"
	@echo ""
	@echo "Install:"
	@echo "  make install           Install to $(BINDIR)"
	@echo "  make install-ascii     Install with ascii-art to $(BINDIR)"
	@echo "  make install-static    Install static binary to $(BINDIR)"
	@echo "  make install-user      Install to ~/.cargo/bin"
	@echo "  make install-ascii-user  Install with ascii-art to ~/.cargo/bin"
	@echo "  make install-static-user Install static binary to ~/.cargo/bin"
	@echo "  make uninstall         Remove from $(BINDIR)"
	@echo "  make uninstall-user    Remove from ~/.cargo/bin"
	@echo ""
	@echo "Run:"
	@echo "  make run               Run debug build"
	@echo "  make run-ascii         Run with ascii-art feature"
	@echo ""
	@echo "Distribution:"
	@echo "  make dist              Create x86_64 release archive"
	@echo "  make dist-all          Create x86_64 and aarch64 release archives"
	@echo ""
	@echo "Development:"
	@echo "  make test              Run tests"
	@echo "  make fmt               Format code"
	@echo "  make fmt-check         Check formatting (no changes)"
	@echo "  make lint              Run clippy lints"
	@echo "  make check             Type-check without building"
	@echo "  make doc               Generate and open documentation"
	@echo "  make pre-commit        Run pre-commit hooks on all files"
	@echo "  make pre-release       Full pre-release validation"
	@echo "  make clean             Remove build artifacts"
	@echo ""
	@echo "Variables:"
	@echo "  PREFIX=$(PREFIX)       Installation prefix"
	@echo "  DESTDIR=               Staging directory for packagers"
