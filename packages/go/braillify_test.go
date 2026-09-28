package braillify

import "testing"

func TestEncodeToUnicode(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"안녕하세요", "⠣⠒⠉⠻⠚⠠⠝⠬"},
		{"상상이상의", "⠇⠶⠇⠶⠕⠇⠶⠺"},
		{"1,000", "⠼⠁⠂⠚⠚⠚"},
		{"ATM", "⠠⠠⠁⠞⠍"},
		{"", ""},
	}

	for _, tt := range tests {
		result, err := EncodeToUnicode(tt.input)
		if err != nil {
			t.Errorf("EncodeToUnicode(%q): unexpected error: %v", tt.input, err)
			continue
		}
		t.Logf("EncodeToUnicode(%q) = %q", tt.input, result)
		if result != tt.expected {
			t.Errorf("EncodeToUnicode(%q) = %q, want %q", tt.input, result, tt.expected)
		}
	}
}

func TestEncode(t *testing.T) {
	result, err := Encode("안녕")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	t.Logf("Encode(%q) = %v", "안녕", result)
	if len(result) == 0 {
		t.Error("expected non-empty byte slice")
	}
}

func TestContextDecidesHowTextIsRead(t *testing.T) {
	tests := []struct {
		context  string
		expected string
	}{
		{"science", "⠴⠏⠠⠕⠠⠓"},
		{"korean", "⠴⠏⠠⠠⠕⠓⠲"},
	}

	for _, tt := range tests {
		unicode, err := EncodeToUnicodeInContext("pOH", tt.context)
		if err != nil {
			t.Fatalf("EncodeToUnicodeInContext(%q): unexpected error: %v", tt.context, err)
		}
		if unicode != tt.expected {
			t.Errorf("EncodeToUnicodeInContext(%q) = %q, want %q", tt.context, unicode, tt.expected)
		}
		font, err := EncodeToBrailleFontInContext("pOH", tt.context)
		if err != nil || font != tt.expected {
			t.Errorf("EncodeToBrailleFontInContext(%q) = %q, %v", tt.context, font, err)
		}
		cells, err := EncodeInContext("pOH", tt.context)
		if err != nil || len(cells) != len([]rune(tt.expected)) {
			t.Errorf("EncodeInContext(%q) = %v, %v", tt.context, cells, err)
		}
	}
}

func TestUnknownContextIsAnError(t *testing.T) {
	if _, err := EncodeToUnicodeInContext("pOH", "chemistry"); err == nil {
		t.Error("expected an error for an unknown context")
	}
}

func TestEncodeToBrailleFont(t *testing.T) {
	result, err := EncodeToBrailleFont("안녕하세요")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	expected := "⠣⠒⠉⠻⠚⠠⠝⠬"
	t.Logf("EncodeToBrailleFont(%q) = %q", "안녕하세요", result)
	if result != expected {
		t.Errorf("EncodeToBrailleFont = %q, want %q", result, expected)
	}
}
