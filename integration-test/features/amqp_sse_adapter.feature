Feature: The service serves as an adapter between RabbitMQ and an SSE stream consumer

  Scenario: A single value is propagated to the SSE stream consumer
    Given the exchange test_exchange exists
    And an SSE stream is opened for routing key "foo"
    And an SSE stream is opened for routing key "bar"
    When "{"test": 1}" is emitted for routing key "foo"
    And "{"test": 2}" is emitted for routing key "bar"
    Then the SSE stream has received the value "{"test": 1}"
    And the SSE stream has received the value "{"test": 2}"

  Scenario: Consumers do not receive other routing keys
    Given the exchange test_exchange exists
    And an SSE stream is opened for routing key "unknown"
    When "{"test": 3}" is emitted for routing key "known"
    Then the SSE stream has not received the value "{"test": 3}"
