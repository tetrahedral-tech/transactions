FROM rust:1.76-bookworm as rust-builder
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

FROM node:21-bookworm-slim as node-builder
WORKDIR /app/
COPY transaction-router/package.json transaction-router/package-lock.json ./
RUN npm install
COPY transaction-router/ . 

FROM node:21-bookworm-slim
RUN apt update && apt install ca-certificates -y
WORKDIR /app
COPY --from=rust-builder /usr/local/cargo/bin/transactions transactions
COPY --from=node-builder /app transaction-router
COPY .env .
CMD ["/app/transactions"]