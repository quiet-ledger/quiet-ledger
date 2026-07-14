.PHONY: help build build-debug test fmt fmt-check clippy check clean doc

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## Build all contracts in release mode
	cargo build --release

build-debug: ## Build all contracts in debug mode
	cargo build

test: ## Run all tests (see CONTRIBUTING.md for a known toolchain caveat)
	cargo test

fmt: ## Format all code
	cargo fmt --all

fmt-check: ## Check formatting without modifying files
	cargo fmt --all -- --check

clippy: ## Run clippy lints (production code only; see CONTRIBUTING.md)
	cargo clippy -- -D warnings

check: fmt-check clippy ## Run available checks (fmt + clippy)
	@echo "Note: 'make test' is currently blocked by an upstream toolchain issue — see CONTRIBUTING.md."

clean: ## Clean build artifacts
	cargo clean

doc: ## Generate documentation
	cargo doc --no-deps
