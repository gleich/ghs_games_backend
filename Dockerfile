# hadolint ignore=DL3007
FROM rust:latest AS builder

# Meta data
LABEL maintainer="email@mattglei.ch"
LABEL description="Backend for the GHS games project"

# File copy
COPY . /usr/src/app
WORKDIR /usr/src/app

# Setup nightly
RUN rustup toolchain install nightly && \
    rustup default nightly

# Binary build
RUN cargo build --release

# Copy of binary to smaller image
# hadolint ignore=DL3006,DL3007
FROM debian:stable-slim
WORKDIR /
COPY --from=builder /usr/src/app/target/release/ghs_games_backend .

# Install needed deps
# hadolint ignore=DL3008
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends libpq5 ca-certificates libssl-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Setting env vars
ENV RUST_LOG info
ENV RUST_BACKTRACE 1

CMD ["./ghs_games_backend"]

