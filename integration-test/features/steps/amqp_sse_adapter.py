from queue import Queue, Empty
from threading import Event as ThreadingEvent
from threading import Thread

import requests
from behave import given, then, when
from behave.runner import Context
from sseclient import SSEClient, Event


class SseConsumer:
    def __init__(self, kill_signal: ThreadingEvent):
        self.__kill_signal = kill_signal

    def consume(self, url: str, queue: Queue):
        messages = SSEClient(url)
        for message in messages:
            if self.__kill_signal.is_set():
                return
            else:
                queue.put(message)


@given('an SSE stream is opened for routing key "{}"')
def step_impl(context: Context, routing_key: str):
    stream = SseConsumer(context.threads_stop_signal)
    thread = Thread(
        target=stream.consume,
        args=(f"http://localhost:8000/events/{routing_key}", context.sse_messages),
    )
    thread.start()


@when('"{}" is emitted for routing key "{}"')
def step_impl(context: Context, message: str, routing_key: str):
    requests.get(f"http://localhost:8000/events/{routing_key}/{message}")


@then('the SSE stream has received the value "{}"')
def step_impl(context: Context, _contents: str):
    found = []
    try:
        while True:
            event: Event = context.sse_messages.get(timeout=0.5)
            # TODO: Connected should become an event content, event remains the
            #  routing key.
            if event:
                break
            else:
                found.append(event.event)
    except Empty:
        raise AssertionError(f"Expected {_contents}, got {found}")
