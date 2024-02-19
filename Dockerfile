FROM rust:1.76-bookworm as builder
WORKDIR /app


# Cache downloaded+built dependencies
COPY Cargo.toml Cargo.lock ./
RUN \
    mkdir /app/src && \
    echo 'fn main() {}' > /app/src/main.rs && \
    cargo build --release && \
    rm -Rvf /app/src

COPY src src
RUN \
    touch src/main.rs && \
    cargo install --path .

RUN apt-get update && apt-get install npm -y
WORKDIR /app/transaction-router
COPY transaction-router/package.json transaction-router/package-lock.json ./
RUN npm install
COPY transaction-router/ . 

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install ca-certificates nodejs -y
COPY --from=builder /usr/local/cargo/bin/transactions transactions
COPY --from=builder /app/transaction-router transaction-router
COPY .env .
CMD ["/app/transactions"]