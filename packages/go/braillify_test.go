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
