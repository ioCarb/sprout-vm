FROM rust:1.79 AS builder

RUN update-ca-certificates

RUN apt update
RUN apt install -y curl libpq-dev libssl-dev libpq-dev librust-openssl-dev librust-openssl-sys-dev
RUN apt -y install libpq5
RUN apt update && apt install -y libprotobuf-dev protobuf-compiler

WORKDIR /rust/src
COPY ./ ./

RUN export CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN export RUST_BACKTRACE=1

RUN cd wasm-server && cargo build --release



FROM ubuntu:22.04

RUN apt update && apt -y install libpq-dev libpq5

WORKDIR /wasmserver/


COPY --from=builder /rust/src/wasm-server/target/release/wasm-server ./

CMD ["/wasmserver/wasm-server"]
