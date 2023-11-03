FROM rust:latest

WORKDIR /app

COPY . .

RUN apt-get update && apt-get install -y \
    libpq-dev

ENV SQLX_OFFLINE=true

RUN cargo build --release --bin

CMD ["./target/release/zero2prod"]
