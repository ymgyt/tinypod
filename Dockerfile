FROM rust:1.57.0-slim-bullseye as builder

RUN cargo new --bin tinypod
WORKDIR /tinypod

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/tinypod*
RUN cargo build --release


FROM debian:bullseye-slim

COPY --from=builder /tinypod/target/release/tinypod .

ENV RUST_LOG=info
CMD ["./tinypod"]
