"""Reads what the household member said into an intent's slots.

ask-sdk-core deprecates `get_slot_value`. Its replacements read the newer
`slotValue` object, which a plain single-value slot need not carry, so this
reads the slot's own `value` through the non-deprecated `get_slot` instead.
"""

from __future__ import annotations

from ask_sdk_core.handler_input import HandlerInput
from ask_sdk_core.utils.request_util import get_slot


def slot_text(handler_input: HandlerInput, slot_name: str) -> str | None:
    """The spoken text in `slot_name`, or None when the slot is missing or empty."""
    slot = get_slot(handler_input, slot_name)
    return None if slot is None else slot.value
