FROM rust:alpine AS builder

# Arguments
ARG CURRENT_COMMIT
ARG CURRENT_BUILD

# Set the working directory
WORKDIR /usr/src/app
COPY . .

# Install necessary build dependencies, including protobuf compiler
RUN apk add --no-cache musl-dev openssl-dev protobuf-dev

# Compile the controller with the wasm-plugins feature
RUN cargo build -p controller --features wasm-plugins --release

# Runtime stage: Use a minimal Alpine image
FROM alpine:latest

# Install necessary runtime dependencies
RUN apk add --no-cache curl unzip

# Set the working directory
WORKDIR /app

# Copy entrypoint script
COPY .github/docker/entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

# Copy the controller binary from the builder stage
COPY --from=builder /usr/src/app/target/release/controller /app/controller

# Expose the port
EXPOSE 12892

# Run the controller
CMD ["./entrypoint.sh"]