FROM rust:latest AS rust-build

RUN apt-get update
RUN apt -y install libc++1

WORKDIR /usr/src/autogram
COPY ./ ./
RUN cargo build --release




FROM debian:bookworm-slim

WORKDIR /autogram

COPY --from=rust-build /usr/src/autogram/target/release/autogram /usr/local/bin/autogram

RUN apt-get update
RUN apt-get install -y openssl ca-certificates libc++1

ENTRYPOINT ["/usr/local/bin/autogram"]
CMD ["start"]