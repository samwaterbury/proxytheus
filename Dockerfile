FROM rust:slim-bookworm AS builder

RUN apt-get update \
    && apt-get install -y libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release

# --- Release Image ---

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y dumb-init ca-certificates openssl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/proxytheus /app/proxytheus

ENTRYPOINT [ "/usr/bin/dumb-init", "--", "/app/proxytheus" ]
