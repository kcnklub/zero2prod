FROM rust:latest

WORKDIR /app

COPY . .

RUN apt-get update && apt-get install -y \
    libpq-dev

RUN cargo build --release

CMD ["./target/release/zero2prod"]
