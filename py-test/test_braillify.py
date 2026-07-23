"""Smoke tests for the braillify Python binding.

The braillify wheel exposes:
  - encode(text) -> bytes
  - translate_to_unicode(text) -> str
  - translate_to_braille_font(text) -> str

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
