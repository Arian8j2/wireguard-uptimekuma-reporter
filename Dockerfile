# builder
FROM rust:1.90-alpine3.22 as builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apk add --no-cache musl-dev

WORKDIR /app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

# amnezia
FROM ghcr.io/arian8j2/wireguard:amnezia as amnezia

# runner
FROM alpine:3.22
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/uptime-reporter /bin/uptime-reporter
COPY --from=amnezia /bin/wg-quick /bin/wg /bin/wireguard-go /bin

RUN apk add --no-cache curl iputils-ping bash

VOLUME /app
WORKDIR /app

CMD ["/bin/uptime-reporter"]
