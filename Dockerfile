FROM ubuntu:latest AS rust-build

RUN apt-get update && apt -y install libc++1 build-essential pkg-config libssl-dev

RUN curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf -o rustup.sh && sh rustup.sh -y && bash -c "echo source $HOME/.cargo/env >> /etc/bash.bashrc"

WORKDIR /usr/src/autogram
COPY ./ ./
RUN cargo build --release
RUN cp $(find target/release/ -name 'libtdjson.so.1.8.29') .




FROM ubuntu:latest

WORKDIR /autogram

COPY --from=rust-build /usr/src/autogram/libtdjson.so.1.8.29 /usr/local/lib/libtdjson.so.1.8.29
COPY --from=rust-build /usr/src/autogram/target/release/autogram /usr/local/bin/autogram

RUN apt-get update && apt-get install -y openssl ca-certificates libc++1

ENTRYPOINT ["/usr/local/bin/autogram"]
CMD ["help"]