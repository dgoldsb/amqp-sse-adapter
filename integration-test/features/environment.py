"""Entrypoint of the ``behave`` testing context."""
from queue import Queue
from threading import Event

from behave.model import Scenario
from behave.runner import Context


def before_all(context: Context):
    context.sse_messages = Queue()


def before_scenario(context: Context, _scenario: Scenario):
    context.threads_stop_signal = Event()
    while not context.sse_messages.empty():
        context.sse_messages.get()


def after_scenario(context: Context, _scenario: Scenario):
    context.threads_stop_signal.set()
