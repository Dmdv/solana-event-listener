FROM rust:1.78.0 AS builder
COPY . /solana-event-listener/

RUN --mount=type=cache,target=/solana-event-listener/target,sharing=locked --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --manifest-path=/solana-event-listener/Cargo.toml \
                --package listener-app \
                --release && \
    mkdir /release && \
    cp /solana-event-listener/target/release/listener-app /release

FROM ubuntu
COPY --from=builder /release/* \
    /solana-event-listener/docker/config.yml /

ENTRYPOINT ["/listener-app", "listen", "--config", "config.yml"]
