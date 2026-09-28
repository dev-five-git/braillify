"""Smoke tests for the braillify Python binding.

The braillify wheel exposes, each with an optional `context` such as "science":
  - encode(text, context=None) -> bytes
  - translate_to_unicode(text, context=None) -> str
  - translate_to_braille_font(text, context=None) -> str

Tests live in `test_*.py` (pytest default discovery pattern) so that pytest 9+
collects them; an `__init__.py` here would mark this directory as a Python
package and skip ordinary test discovery.
"""

import pytest
import braillify


@pytest.mark.parametrize(
    "input, expected",
    [
        # Korean (한글 음절)
        ("안녕하세요", "⠣⠒⠉⠻⠚⠠⠝⠬"),
        # English (lowercase — no grade indicator)
        ("hello", "⠓⠑⠇⠇⠕"),
        # English (single capital → ⠠ prefix)
        ("Hello", "⠠⠓⠑⠇⠇⠕"),
        # English (full uppercase → ⠠⠠ double cap)
        ("BMI", "⠠⠠⠃⠍⠊"),
        # Number (⠼ digit indicator)
        ("1234", "⠼⠁⠃⠉⠙"),
    ],
)
def test_translate_to_unicode(input: str, expected: str) -> None:
    assert braillify.translate_to_unicode(input) == expected


def test_encode_returns_bytes() -> None:
    out = braillify.encode("안녕")
    assert isinstance(out, (bytes, bytearray))
    assert len(out) > 0


def test_translate_to_braille_font_returns_str() -> None:
    out = braillify.translate_to_braille_font("안녕")
    assert isinstance(out, str)
    assert len(out) > 0


@pytest.mark.parametrize(
    "context, expected",
    [
        ("science", "⠴⠏⠠⠕⠠⠓"),
        ("korean", "⠴⠏⠠⠠⠕⠓⠲"),
    ],
)
def test_context_decides_how_text_is_read(context: str, expected: str) -> None:
    assert braillify.translate_to_unicode("pOH", context=context) == expected
    assert braillify.translate_to_braille_font("pOH", context) == expected
    assert len(braillify.encode("pOH", context)) == len(expected)


def test_unknown_context_raises_value_error() -> None:
    with pytest.raises(ValueError):
        braillify.translate_to_unicode("pOH", context="chemistry")
