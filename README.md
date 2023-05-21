# amqp-sse-adapter

A small pet project to translate events emitted on an AMQP exchange to an SSE stream.
This is useful if a frontend application is interested in events emitted on the exchange.

## Running locally

The project is build using `cargo`, executing `cargo build` in the repository root will compile the project.
The application can be spun up locally with `docker compose up --build`.
This will build an image in two stages following the steps in the [Dockerfile](./Dockerfile), and spin up a local RabbitMQ container and the `amqp-sse-adapter`.
For testing, there are integration tests included written in Python.
To run these, navigate to [integration-test](./integration-test) and run `bash run.sh`.
