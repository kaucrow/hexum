# =======================
#  Chef (Caching Tool)
# =======================
FROM rust:alpine AS chef
WORKDIR /usr/src/app

RUN cargo install cargo-chef

# ================================
#  Planner (Analyzes Workspace)
# ================================
FROM chef AS planner
COPY . .
# This analyzes all workspace crates and creates a dependency recipe
RUN cargo chef prepare --recipe-path recipe.json

# ==========================
#  Builder (Compiles App)
# ==========================
FROM chef AS builder
COPY --from=planner /usr/src/app/recipe.json recipe.json

# Build just the dependencies first so Docker caches this layer
RUN cargo chef cook --release --recipe-path recipe.json

# Now copy the code
COPY . .

# Compile just the api crate
RUN cargo build --release -p api

# =======================
#  Runtime Environment
# =======================
FROM alpine:latest
WORKDIR /app

RUN apk add --no-cache redis ca-certificates

# Copy the compiled executable
COPY --from=builder /usr/src/app/target/release/api /app/api

# Copy the global configuration from the workspace root
COPY --from=builder /usr/src/app/config /app/config

# Copy the crate-specific assets into their namespaced folders
COPY --from=builder /usr/src/app/platform/postgres /app/platform/postgres

# TODO: Add business crate
# COPY --from=builder /usr/src/app/business/postgres /app/business/postgres

# Run the compiled binary
CMD ["/app/api"]