# frozen_string_literal: true

require "minitest/autorun"
require "braillify"

class BraillifyTest < Minitest::Test
  def test_translate_to_unicode_returns_braille_unicode
    out = Braillify.translate_to_unicode("안녕하세요")
    refute_empty out
    out.each_char do |c|
      assert_includes 0x2800..0x28FF, c.ord, "non-braille char: #{c.inspect}"
    end
  end

  def test_encode_returns_binary_string
    out = Braillify.encode("안녕")
    assert_equal Encoding::ASCII_8BIT, out.encoding
    refute_empty out
  end

  def test_encode_bytes_match_unicode_codepoints
    text = "안녕하세요"
    unicode = Braillify.translate_to_unicode(text)
    bytes = Braillify.encode(text)
    assert_equal unicode.chars.map { |c| c.ord - 0x2800 }, bytes.bytes
  end

  def test_translate_to_braille_font
    refute_empty Braillify.translate_to_braille_font("hi")
  end

  def test_unsupported_input_raises_argument_error
    assert_raises(ArgumentError) { Braillify.translate_to_unicode("😀") }
    assert_raises(ArgumentError) { Braillify.encode("😀") }
    assert_raises(ArgumentError) { Braillify.translate_to_braille_font("😀") }
  end

  def test_version_matches_cargo_toml
    assert_match(/\A\d+\.\d+\.\d+\z/, Braillify::VERSION)
  end
end
