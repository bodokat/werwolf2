FROM rust:latest AS builder

# Install dependencies

RUN USER=root cargo new --bin werwolf

WORKDIR /werwolf

COPY ./server/Cargo.lock ./Cargo.lock
COPY ./server/Cargo.toml ./Cargo.toml

RUN cargo build --release

RUN rm src/*.rs

# Build

COPY ./server/src ./src
RUN rm ./target/release/deps/werwolf*

RUN cargo build --release

# ---

FROM debian:bullseye-slim

COPY --from=builder /werwolf/target/release/werwolf .
COPY ./web/dist ./static

CMD ["./werwolf"]