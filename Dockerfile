FROM docker.io/library/rust:1.85-alpine as builder

WORKDIR /app

COPY . .

# Install crti.o
RUN apk add --no-cache musl-dev && \
    rustup target add x86_64-unknown-linux-musl && \
    cargo build --release --target x86_64-unknown-linux-musl

FROM docker.io/library/alpine:3.20
WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/patch-release-me /usr/local/bin/patch-release-me

ENTRYPOINT ["patch-release-me"]

