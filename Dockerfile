# Stage 1: Build the Rust app
FROM rust:1.82.0-bookworm AS builder

# Set the working directory inside the container
WORKDIR /app

# Install required system dependencies for SQLx, OpenSSL (for TLS), etc.
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    postgresql-client

# Install Node.js and Tailwind CSS
RUN apt-get update && \
    apt-get install -y curl && \
    curl -fsSL https://deb.nodesource.com/setup_16.x | bash - && \
    apt-get install -y nodejs && \
    npm install -D tailwindcss@3.4.14 postcss@8.4.47 autoprefixer@10.4.20 daisyui@4.12.23

# Accept DATABASE_URL as a build argument
ARG DATABASE_URL
ENV DATABASE_URL=${DATABASE_URL}

# Copy the source code (just for dependency resolution)
COPY . .

# Build the styles using Tailwind CSS
RUN npx tailwindcss -i ./assets/styles.css -o ./public/styles.css --minify

# This step will cache dependencies to speed up builds
RUN cargo fetch

# Run tests - There was a problem with production database, that would not give superuser access to create fixtures with SQLx library.
# RUN cargo test --locked --all-targets

# Compile the project in release mode
RUN cargo build --release

# At this point, we have the compiled binary

# Stage 2: Create the final runtime image
FROM debian:bookworm-slim

# Install system dependencies needed at runtime
RUN apt-get update && apt-get install -y \
    libssl-dev \
    libpq-dev \
    postgresql-client && \
    apt-get clean

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the build stage
COPY --from=builder /app/target/release/pv281-giglog .
COPY --from=builder /app/public/styles.css ./public/styles.css

# Expose the port your Axum app will run on
EXPOSE 3000

# Run the application
CMD ["./pv281-giglog"]