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

.PHONY: all build rebuild release static static-x86 static-arm install uninstall \
        clean dist test fmt lint check pre-commit help

# Default target: show help
all: help

# Full rebuild from scratch (clears all cargo cache)
rebuild:
	$(CARGO) clean
	$(RM) -r $(HOME)/.cargo/registry/cache
	$(RM) -r $(HOME)/.cargo/git/checkouts
	$(CARGO) build --release

# Development build
build:
	$(CARGO) build

# Optimized release build
release:
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

# Install to system (default: /usr/local/bin)
install: release
	$(INSTALL) -d $(DESTDIR)$(BINDIR)
	$(INSTALL) -m 755 $(RELEASE_DIR)/$(BINARY) $(DESTDIR)$(BINDIR)/$(BINARY)
	@echo "Installed $(BINARY) to $(DESTDIR)$(BINDIR)"

# Install static binary
install-static: static-x86
	$(INSTALL) -d $(DESTDIR)$(BINDIR)
	$(INSTALL) -m 755 $(STATIC_X86_DIR)/$(BINARY) $(DESTDIR)$(BINDIR)/$(BINARY)
	@echo "Installed static $(BINARY) to $(DESTDIR)$(BINDIR)"

# Install to ~/.cargo/bin (user install)
install-user: release
	$(CARGO) install --path .
	@echo "Installed $(BINARY) to ~/.cargo/bin"

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

# Run tests
test:
	$(CARGO) test

# Format code
fmt:
	$(CARGO) fmt

# Lint code
lint:
	$(CARGO) clippy -- -D warnings

# Run pre-commit on all files
pre-commit:
	python3 -m pre_commit run --all-files

# Check without building
check:
	$(CARGO) check

# Clean build artifacts
clean:
	$(CARGO) clean
	$(RM) -r $(DIST_DIR)

# Show help
help:
	@echo "trex $(VERSION) - tmux session manager"
	@echo ""
	@echo "Build targets:"
	@echo "  make              Show this help"
	@echo "  make build        Build debug binary"
	@echo "  make release      Build optimized release binary"
	@echo "  make rebuild      Clean all cache and rebuild from scratch"
	@echo "  make static       Build static x86_64 Linux binary (musl)"
	@echo "  make static-x86   Build static x86_64 Linux binary (musl)"
	@echo "  make static-arm   Build static aarch64 Linux binary (musl)"
	@echo ""
	@echo "Install targets:"
	@echo "  make install      Install to $(BINDIR) (may need sudo)"
	@echo "  make install-static  Install static binary to $(BINDIR)"
	@echo "  make install-user Install to ~/.cargo/bin"
	@echo "  make uninstall    Remove from $(BINDIR)"
	@echo "  make uninstall-user  Remove from ~/.cargo/bin"
	@echo ""
	@echo "Distribution:"
	@echo "  make dist         Create x86_64 release archive"
	@echo "  make dist-all     Create x86_64 and aarch64 release archives"
	@echo ""
	@echo "Development:"
	@echo "  make test         Run tests"
	@echo "  make fmt          Format code"
	@echo "  make lint         Run clippy lints"
	@echo "  make check        Check code without building"
	@echo "  make pre-commit   Run pre-commit hooks on all files"
	@echo "  make clean        Remove build artifacts"
	@echo ""
	@echo "Variables:"
	@echo "  PREFIX=$(PREFIX)  Installation prefix"
	@echo "  DESTDIR=          Staging directory for packagers"
