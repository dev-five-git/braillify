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
