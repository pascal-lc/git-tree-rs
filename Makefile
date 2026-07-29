# git-tree (Rust) Makefile
# ==============================================================================

PREFIX     ?= $(HOME)/.local
BINDIR     ?= $(PREFIX)/bin
MANDIR     ?= $(PREFIX)/share/man/man1
VERSION    := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
TARGET     := target/release/git-tree

.DEFAULT_GOAL := help

# ---- targets -----------------------------------------------------------------

.PHONY: help
help: ## Show this help
	@echo "git-tree v$(VERSION) (Rust)"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| sort \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}'

.PHONY: build
build: ## Build in release mode
	@cargo build --release
	@echo "Built $(TARGET)"

.PHONY: install
install: build ## Install git-tree binary and man page
	@install -d "$(BINDIR)"
	@install -m 0755 "$(TARGET)" "$(BINDIR)/git-tree"
	@install -d "$(MANDIR)"
	@install -m 0644 man/git-tree.1 "$(MANDIR)/git-tree.1"
	@echo "Installed git-tree v$(VERSION)"
	@echo "  binary → $(BINDIR)/git-tree"
	@echo "  man    → $(MANDIR)/git-tree.1"
	@echo ""
	@echo "Try: git tree --help"

.PHONY: install-bin
install-bin: build ## Install binary only (no man page)
	@install -d "$(BINDIR)"
	@install -m 0755 "$(TARGET)" "$(BINDIR)/git-tree"
	@echo "Installed git-tree v$(VERSION) → $(BINDIR)/git-tree"

.PHONY: uninstall
uninstall: ## Remove binary and man page
	@rm -f "$(BINDIR)/git-tree"
	@rm -f "$(MANDIR)/git-tree.1"
	@echo "Removed git-tree"

.PHONY: check
check: ## Run clippy lints
	@cargo clippy -- -D warnings 2>/dev/null \
		&& echo "Clippy: OK" \
		|| cargo clippy

.PHONY: test
test: ## Run cargo test
	@cargo test

.PHONY: version
version: ## Print current version
	@echo "git-tree v$(VERSION)"

.PHONY: bump-version
bump-version: ## Bump version (usage: make bump-version NEW=1.1.0)
	@[ -n "$(NEW)" ] || (echo "Usage: make bump-version NEW=<version>"; exit 1)
	@sed -i 's/^version = ".*"/version = "$(NEW)"/' Cargo.toml
	@echo "Version bumped: $(VERSION) → $(NEW)"

.PHONY: clean
clean: ## Remove build artifacts
	@cargo clean
	@echo "Cleaned build artifacts"

.PHONY: distclean
distclean: clean ## Also remove cached crates
	@rm -rf ~/.cargo/registry/cache/*
	@echo "Cleaned crate cache"
