# =======================
#   Builder
# =======================
FROM rust:1.88-alpine AS builder
WORKDIR /usr/src/app

RUN apk add --no-cache \
    musl-dev \
    build-base \
    postgresql-dev

# Copy only the dependency manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy source file to satisfy Cargo's compiler expectations
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies first. Docker caches these dependencies
RUN cargo build --release

# Tear down the dummy source file
RUN rm -rf src/

# Copy the app's code
COPY . .

# Compile just the app layer
RUN cargo build --release

# =======================
#   Runtime environment
# =======================
FROM alpine:latest
WORKDIR /app

RUN apk add --no-cache \
    libpq \
    redis \
    ca-certificates

COPY --from=builder /usr/src/app/target/release/hexum /app/hexum

COPY --from=builder /usr/src/app/config /app/config
COPY --from=builder /usr/src/app/postgres /app/postgres

CMD ["/app/hexum"]