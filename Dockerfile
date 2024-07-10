####################################################################################################
## Builder
####################################################################################################
FROM rust:1.79 AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN apt install --no-install-recommends --assume-yes protobuf-compiler
RUN update-ca-certificates

WORKDIR /halo2-server

COPY ./ .

RUN cargo build --release

####################################################################################################
## Final image
####################################################################################################
FROM ubuntu:24.04

WORKDIR /halo2-server

# Copy our build
COPY --from=builder /halo2-server/target/release/halo2-server ./

CMD ["/halo2-server/halo2-server"]
