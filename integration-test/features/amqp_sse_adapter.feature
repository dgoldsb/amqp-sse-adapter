Feature: The service serves as an adapter between RabbitMQ and an SSE stream consumer

  Scenario: A single value is propagated to the SSE stream consumer
    Given the exchange test_exchange exists
    And an SSE stream is opened for routing key "foo"
    When "bar" is emitted for routing key "foo"
    Then the SSE stream has received the value "bar"

    # TODO: Test that unsubscribing does not lead to a dangling queue.
    # TODO: Test a fanout, multiple streams that all receive one message, probably need to name the connections.
    # TODO: Test many streams at once for different routing keys.Ability:
    # TODO: Run these tests in a GitHub action.
