FROM debian:bullseye-slim

RUN apt-get update -y && apt-get upgrade -y

RUN apt-get install -y curl make build-essential grub-pc xorriso nasm qemu-system-x86
    
WORKDIR /kfs-1
    
COPY . /kfs-1/.

RUN mkdir -p /root/.cargo && cp /kfs-1/config.toml /root/.cargo/config.toml
    
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    . "$HOME/.cargo/env" && \
    rustup toolchain install nightly-x86_64-unknown-linux-gnu && \
    rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu

ENTRYPOINT [ "/bin/bash" ]
