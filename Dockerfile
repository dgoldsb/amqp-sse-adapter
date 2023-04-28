FROM rust:1.69 as builder

WORKDIR /usr/src/amqp-sse-adapter
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim

COPY --from=builder /usr/local/cargo/bin/amqp-sse-adapter /usr/local/bin/amqp-sse-adapter

CMD ["amqp-sse-adapter"]
