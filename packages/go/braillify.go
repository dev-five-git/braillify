package braillify

// Encode converts Korean text to braille byte representation.
func Encode(text string) ([]byte, error) {
	return cEncode(text)
}

// EncodeToUnicode converts Korean text to braille Unicode string.
func EncodeToUnicode(text string) (string, error) {
	return cEncodeToUnicode(text)
}

// EncodeToBrailleFont converts Korean text to braille font string.
func EncodeToBrailleFont(text string) (string, error) {
	return cEncodeToBrailleFont(text)
}

// EncodeInContext converts text read in a named context ("science", "math",
// "korean", ...) to braille bytes. The context decides input whose print shape
// alone does not; an unknown context is an error.
func EncodeInContext(text, context string) ([]byte, error) {
	return cEncodeInContext(text, context)
}

// EncodeToUnicodeInContext converts text read in a named context to a braille
// Unicode string.
func EncodeToUnicodeInContext(text, context string) (string, error) {
	return cEncodeToUnicodeInContext(text, context)
}

// EncodeToBrailleFontInContext converts text read in a named context to a
// braille font string.
func EncodeToBrailleFontInContext(text, context string) (string, error) {
	return cEncodeToBrailleFontInContext(text, context)
}
