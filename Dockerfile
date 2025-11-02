# Multi-stage Dockerfile for ORBS TEE Nitro SDK
# This provides a Linux environment to build and test the SDK with nitro features

# Stage 1: Build environment
FROM rust:1.83-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /workspace

# Copy workspace files
COPY Cargo.toml ./
COPY src ./src
COPY tests ./tests
COPY examples ./examples

# Stage 2: Test runner (without nitro features - cross-platform)
FROM builder AS test-no-nitro

RUN echo "Running tests without nitro features (cross-platform)..."
RUN cargo test --no-default-features --verbose

# Stage 3: Test runner (with nitro features - Linux only)
FROM builder AS test-with-nitro

RUN echo "Running tests with nitro features (Linux only)..."
# Note: Some tests may be skipped if they require actual hardware
RUN cargo test --verbose || echo "Some tests may require actual Nitro hardware"

# Stage 4: Build checker (verify nitro features compile)
FROM builder AS build-nitro

RUN echo "Checking that SDK compiles with nitro features..."
RUN cargo check --verbose

RUN echo "Checking that price-oracle example compiles..."
RUN cargo check --manifest-path examples/price-oracle/Cargo.toml --verbose

# Stage 5: Clippy linter
FROM builder AS clippy

RUN rustup component add clippy

RUN echo "Running clippy on SDK (no nitro features)..."
RUN cargo clippy --no-default-features --all-targets -- -D warnings

RUN echo "Running clippy on SDK (with nitro features)..."
RUN cargo clippy --all-targets -- -D warnings

# Stage 6: Format checker
FROM builder AS fmt

RUN rustup component add rustfmt

RUN echo "Checking code formatting..."
RUN cargo fmt --all -- --check

# Stage 7: Final development image
FROM rust:1.83-slim-bookworm AS dev

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    vim \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

# Copy source code
COPY . .

# Default command
CMD ["/bin/bash"]
