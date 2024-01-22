FROM rust:1.75-bullseye as builder
WORKDIR /usr/src/transactions
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install ca-certificates -y
COPY --from=builder /usr/local/cargo/bin/transactions /usr/local/bin/transactions
COPY --from=builder /usr/src/transactions/.env .
COPY --from=builder /usr/src/transactions/coins.json .
CMD ["transactions"]
