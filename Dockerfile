# builder
FROM rust:1.72-alpine3.17 as builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apk add --no-cache musl-dev

WORKDIR /app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

# runner
FROM alpine:3.19.2
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/uptime-reporter /bin/uptime-reporter

RUN apk add --no-cache curl iputils-ping wireguard-tools

VOLUME /app
WORKDIR /app

CMD ["/bin/uptime-reporter"]
