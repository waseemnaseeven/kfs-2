FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
      build-essential curl ca-certificates git python3 \
      clang lld nasm grub-pc-bin xorriso mtools qemu-system-x86 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

ENV PATH="/root/.cargo/bin:${PATH}"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain nightly && \
    rustup toolchain install nightly-x86_64-unknown-linux-gnu && \
    rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu


