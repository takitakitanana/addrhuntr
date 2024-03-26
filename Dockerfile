FROM rust:alpine as builder
RUN apk add --no-cache build-base openssl-dev
WORKDIR /app/addrhuntr
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo build --release && \
    strip target/release/addrhuntr

FROM alpine:latest
RUN apk add --no-cache libgcc
COPY --from=builder /app/addrhuntr/target/release/addrhuntr /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/addrhuntr"]