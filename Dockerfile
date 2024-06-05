FROM debian@sha256:82f8da149d6d567c74564cccd6f355fb5ade42a958e4cde10a1100eaeb24d42e AS tdlib-build

WORKDIR /usr/src/

RUN apt-get update
RUN apt-get install -y gcc pkg-config cmake g++ gperf libssl-dev zlib1g-dev git
RUN git clone https://github.com/tdlib/td.git
WORKDIR /usr/src/td/
RUN git checkout 2589c3fd46925f5d57e4ec79233cd1bd0f5d0c09
RUN mkdir build
RUN /usr/src/td/build/
RUN cmake -DCMAKE_BUILD_TYPE=Release ..
RUN cmake --build .
CMD exec /usr/local/bin/autogram




FROM rust:latest AS rust-build

RUN apt-get update
RUN apt-get install -y gcc pkg-config cmake g++ gperf libssl-dev zlib1g-dev

COPY --from=tdlib-build /usr/src/td/build/pkgconfig/* /usr/lib/pkgconfig/
COPY --from=tdlib-build /usr/src/td/build/libtdjson.so* /usr/local/lib/

RUN ldconfig

WORKDIR /usr/src/autogram
COPY ./ ./
RUN cargo build --release




FROM debian:bookworm-slim

WORKDIR /autogram

COPY --from=rust-build /usr/src/autogram/target/release/autogram /usr/local/bin/autogram

RUN apt-get update
RUN apt-get install -y openssl ca-certificates

CMD exec /usr/local/bin/autogram