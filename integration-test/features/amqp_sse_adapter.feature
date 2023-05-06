Feature: The service serves as an adapter between RabbitMQ and an SSE stream consumer

  Scenario: A single value is propagated to the SSE stream consumer
    Given an SSE stream is opened for routing key "foo"
    When "bar" is emitted for routing key "foo"
    Then the SSE stream has received the value "bar"
