package io.github.kdyann.braillify;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.Callable;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import org.junit.jupiter.api.Test;

class BraillifyTest {
    @Test
    void encodeAndUnicodeRepresentTheSameCells() {
        byte[] cells = Braillify.encode("안녕하세요");
        String unicode = Braillify.translateToUnicode("안녕하세요");

        int[] codePoints = unicode.codePoints().toArray();
        assertEquals(cells.length, codePoints.length);
        for (int index = 0; index < cells.length; index++) {
            assertEquals(0x2800 + Byte.toUnsignedInt(cells[index]), codePoints[index]);
        }
    }

    @Test
    void brailleFontResultMatchesTheCoreUnicodeRepresentation() {
        assertEquals(Braillify.translateToUnicode("점자"),
                Braillify.translateToBrailleFont("점자"));
    }

    @Test
    void rejectsUnsupportedCharactersAsPublicErrors() {
        assertThrows(BraillifyException.class, () -> Braillify.encode("😀"));
    }

    @Test
    void rejectsNullBeforeEnteringNativeCode() {
        assertThrows(NullPointerException.class, () -> Braillify.encode(null));
        assertThrows(NullPointerException.class, () -> Braillify.translateToUnicode(null));
        assertThrows(NullPointerException.class, () -> Braillify.translateToBrailleFont(null));
    }

    @Test
    void rejectsMalformedUtf16WithoutReplacement() {
        assertThrows(BraillifyException.class, () -> Braillify.encode("\uD800"));
        assertThrows(BraillifyException.class, () -> Braillify.encode("\uDC00"));
        assertThrows(BraillifyException.class, () -> Braillify.encode("\uDC00\uD800"));
    }

    @Test
    void preservesEmbeddedNulUntilTheCoreRejectsIt() {
        assertThrows(BraillifyException.class, () -> Braillify.encode("a\u0000b"));
    }

    @Test
    void nativePanicsBecomeInternalErrors() {
        NativeLibraryLoader.ensureLoaded();
        assertThrows(BraillifyInternalException.class, Braillify::panicForTestingNative);
    }

    @Test
    void callsFromSeveralThreadsDoNotLeakEncoderState() throws Exception {
        byte[] expected = Braillify.encode("안녕 hello 123");
        ExecutorService executor = Executors.newFixedThreadPool(8);
        try {
            List<Callable<byte[]>> calls = new ArrayList<>();
            for (int index = 0; index < 128; index++) {
                calls.add(() -> Braillify.encode("안녕 hello 123"));
            }
            List<Future<byte[]>> futures = executor.invokeAll(calls);
            for (Future<byte[]> future : futures) {
                assertArrayEquals(expected, future.get());
            }
        } finally {
            executor.shutdownNow();
        }
    }

    @Test
    void unicodeOutputContainsOnlyBrailleCodePoints() {
        assertTrue(Braillify.translateToUnicode("hello 안녕 123").codePoints()
                .allMatch(value -> value >= 0x2800 && value <= 0x28ff));
    }
}
