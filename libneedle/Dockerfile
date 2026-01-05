# Author : Eshan Roy <eshanized@proton.me>
# SPDX-License-Identifier: MIT

FROM rust:1.83-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release --bin needle

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash needle
USER needle

COPY --from=builder /app/target/release/needle /usr/local/bin/needle

EXPOSE 3000 2222

CMD ["needle"]
