package braillify

/*
#cgo darwin,amd64 LDFLAGS: -L${SRCDIR}/libs/darwin-amd64 -lbraillify_go -lm -lpthread
#cgo darwin,arm64 LDFLAGS: -L${SRCDIR}/libs/darwin-arm64 -lbraillify_go -lm -lpthread
#cgo linux,amd64 LDFLAGS: -L${SRCDIR}/libs/linux-amd64 -lbraillify_go -lm -lpthread -ldl
#cgo linux,arm64 LDFLAGS: -L${SRCDIR}/libs/linux-arm64 -lbraillify_go -lm -lpthread -ldl
#cgo windows,amd64 LDFLAGS: -L${SRCDIR}/libs/windows-amd64 -lbraillify_go -lntdll -lws2_32 -lbcrypt -ladvapi32 -luserenv

#include <stdlib.h>
#include <stdint.h>
#include <stddef.h>

extern uint8_t* braillify_encode(const char* text, size_t* out_len);
extern char* braillify_encode_to_unicode(const char* text);
extern char* braillify_encode_to_braille_font(const char* text);
extern char* braillify_get_last_error();
extern void braillify_free_string(char* ptr);
extern void braillify_free_bytes(uint8_t* ptr, size_t len);
*/
import "C"

import (
	"errors"
	"runtime"
	"unsafe"
)

func cEncode(text string) ([]byte, error) {
	runtime.LockOSThread()
	defer runtime.UnlockOSThread()

	cText := C.CString(text)
	defer C.free(unsafe.Pointer(cText))

	var outLen C.size_t
	result := C.braillify_encode(cText, &outLen)
	if result == nil {
		return nil, getLastError()
	}
	defer C.braillify_free_bytes(result, outLen)

	return C.GoBytes(unsafe.Pointer(result), C.int(outLen)), nil
}

func cEncodeToUnicode(text string) (string, error) {
	runtime.LockOSThread()
	defer runtime.UnlockOSThread()

	cText := C.CString(text)
	defer C.free(unsafe.Pointer(cText))

	result := C.braillify_encode_to_unicode(cText)
	if result == nil {
		return "", getLastError()
	}
	defer C.braillify_free_string(result)

	return C.GoString(result), nil
}

func cEncodeToBrailleFont(text string) (string, error) {
	runtime.LockOSThread()
	defer runtime.UnlockOSThread()

	cText := C.CString(text)
	defer C.free(unsafe.Pointer(cText))

	result := C.braillify_encode_to_braille_font(cText)
	if result == nil {
		return "", getLastError()
	}
	defer C.braillify_free_string(result)

	return C.GoString(result), nil
}

func getLastError() error {
	errPtr := C.braillify_get_last_error()
	if errPtr == nil {
		return errors.New("braillify: unknown error")
	}
	defer C.braillify_free_string(errPtr)
	return errors.New(C.GoString(errPtr))
}
