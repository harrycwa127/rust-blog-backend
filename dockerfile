# Multi-stage build for Rust backend
FROM rustlang/rust:nightly-alpine AS builder

# Install build dependencies (cached layer)
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig curl

# Create app directory
WORKDIR /app

# Copy manifest files first (for dependency caching)
COPY Cargo.toml Cargo.lock ./
COPY migration/Cargo.toml migration/Cargo.lock ./migration/

# Create dummy source files to cache dependencies
RUN mkdir -p src migration/src && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > migration/src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release
RUN rm -rf src migration/src

# Copy real source code
COPY src ./src
COPY migration/src ./migration/src

# Build the application (only app code, dependencies are cached)
RUN touch src/main.rs migration/src/main.rs && \
    cargo build --release

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache ca-certificates openssl wget

# Create non-root user
RUN addgroup -g 1001 -S appgroup && \
    adduser -S appuser -u 1001 -G appgroup

# Create app directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/blog .

# Change ownership to non-root user
RUN chown -R appuser:appgroup /app
USER appuser

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1

# Start the application
CMD ["./blog"]