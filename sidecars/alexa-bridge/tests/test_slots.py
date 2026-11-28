"""Tests for reading a slot's spoken text out of an intent request."""

import warnings

from ask_sdk_core.handler_input import HandlerInput
from ask_sdk_model import Intent, IntentRequest, RequestEnvelope, Slot

from alexa_bridge.slots import slot_text


def handler_input(**slots):
    """A HandlerInput for an IntentRequest with these slots."""
    filled = {key: Slot(name=key, value=value) for key, value in slots.items()}
    request = IntentRequest(request_id="req-1", intent=Intent(name="AddItemIntent", slots=filled))
    return HandlerInput(request_envelope=RequestEnvelope(request=request))


def test_a_filled_slot_gives_its_text():
    assert slot_text(handler_input(item="milk"), "item") == "milk"


def test_an_unfilled_slot_gives_none():
    assert slot_text(handler_input(item=None), "item") is None


def test_a_missing_slot_gives_none():
    assert slot_text(handler_input(), "quantity") is None


def test_reading_a_slot_uses_no_deprecated_sdk_call():
    with warnings.catch_warnings():
        warnings.simplefilter("error", DeprecationWarning)
        slot_text(handler_input(item="milk"), "item")
