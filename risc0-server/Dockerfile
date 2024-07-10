FROM rust:1.79 AS builder

RUN update-ca-certificates

RUN apt update
RUN apt install -y curl libpq-dev libssl-dev libpq-dev librust-openssl-dev librust-openssl-sys-dev
RUN apt -y install libpq5
RUN apt update && apt install -y libprotobuf-dev protobuf-compiler

RUN cargo install diesel_cli --no-default-features --features postgres --version 2.1.0

RUN curl -L https://foundry.paradigm.xyz | bash
RUN /root/.foundry/bin/foundryup
RUN cp /root/.foundry/bin/* /usr/bin/

WORKDIR /rust/src
COPY ./ ./

RUN export CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN export RUST_BACKTRACE=1

RUN cd risc0-server && cargo build --release



FROM rust:1.79

RUN apt update && apt -y install libpq-dev libpq5

RUN cargo install cargo-binstall
RUN echo yes | cargo binstall cargo-risczero
RUN cargo risczero install


WORKDIR /risc0server/


COPY --from=builder /rust/src/risc0-server/target/release/risc0-server ./
COPY ./risc0-server/Cargo.toml /risc0server/
COPY ./risc0-server/diesel.toml /risc0server/
COPY ./risc0-server/verify_contract_abi.json /risc0server/

#RUN diesel setup
#RUN diesel migration generate risc0-server
#RUN diesel migration run

CMD ["/risc0server/risc0-server"]
