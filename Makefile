# Makefile for ORBS TEE Nitro SDK
# Provides convenient commands for building and testing

.PHONY: help test test-no-nitro test-with-nitro build check clippy fmt docker-build docker-test docker-shell clean

# Default target
help:
	@echo "ORBS TEE Nitro SDK - Available Commands:"
	@echo ""
	@echo "Local (macOS/cross-platform):"
	@echo "  make test              - Run tests without nitro features"
	@echo "  make check             - Check compilation without nitro features"
	@echo "  make clippy            - Run clippy linter"
	@echo "  make fmt               - Format code"
	@echo "  make fmt-check         - Check code formatting"
	@echo ""
	@echo "Docker (Linux environment):"
	@echo "  make docker-build      - Build Docker image"
	@echo "  make docker-test       - Run all tests in Docker"
	@echo "  make docker-test-nitro - Run tests with nitro features in Docker"
	@echo "  make docker-check      - Check compilation with nitro features in Docker"
	@echo "  make docker-clippy     - Run clippy in Docker"
	@echo "  make docker-fmt        - Check formatting in Docker"
	@echo "  make docker-shell      - Open bash shell in Docker container"
	@echo "  make docker-all        - Run all Docker checks (test, clippy, fmt)"
	@echo ""
	@echo "Cleanup:"
	@echo "  make clean             - Remove build artifacts"

# Local commands (macOS/cross-platform)
test:
	cargo test --no-default-features --verbose

test-no-nitro:
	cargo test --no-default-features --verbose

check:
	cargo check --no-default-features --verbose

clippy:
	cargo clippy --no-default-features --all-targets -- -D warnings

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

# Docker commands (Linux environment)
docker-build:
	docker build -t orbs-tee-nitro:latest .

docker-test:
	docker build --target test-no-nitro -t orbs-tee-nitro:test-no-nitro .

docker-test-nitro:
	docker build --target test-with-nitro -t orbs-tee-nitro:test-with-nitro .

docker-check:
	docker build --target build-nitro -t orbs-tee-nitro:build-nitro .

docker-clippy:
	docker build --target clippy -t orbs-tee-nitro:clippy .

docker-fmt:
	docker build --target fmt -t orbs-tee-nitro:fmt .

docker-shell:
	docker build --target dev -t orbs-tee-nitro:dev .
	docker run -it --rm -v $$(pwd):/workspace orbs-tee-nitro:dev /bin/bash

docker-all: docker-test docker-test-nitro docker-check docker-clippy docker-fmt
	@echo ""
	@echo "✅ All Docker checks passed!"

# Cleanup
clean:
	cargo clean
	rm -rf target/
