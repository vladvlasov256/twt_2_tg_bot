FROM rust:1.64 as builder

WORKDIR /twt_2_tg_bot

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

RUN cargo build --release

###

FROM debian:buster-slim

RUN apt-get update
RUN apt-get install -y libpq-dev

COPY --from=builder /twt_2_tg_bot/target/release/twt_2_tg_bot /usr/bin

ENV RUST_BACKTRACE=1
ENV RUST_LOG=debug

RUN apt-get update -y && apt-get install -y ca-certificates

CMD ["twt_2_tg_bot"]