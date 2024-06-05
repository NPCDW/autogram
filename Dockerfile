FROM rust:latest AS rust-build

WORKDIR /usr/src/autogram
COPY ./ ./
WORKDIR /usr/src/autogram/src
RUN cargo build --release



FROM debian:bookworm-slim

WORKDIR /autogram

COPY --from=rust-build /usr/src/autogram/target/release/autogram /usr/local/bin/autogram

RUN apt-get update
RUN apt-get install -y openssl ca-certificates

CMD exec /usr/local/bin/autogram