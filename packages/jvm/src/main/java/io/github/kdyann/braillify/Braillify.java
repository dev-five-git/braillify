package io.github.kdyann.braillify;

import java.util.Objects;

/** Korean text-to-braille conversion backed by the braillify Rust engine. */
public final class Braillify {
    private Braillify() {
    }

    /**
     * Converts text to braille cell values (0 through 255).
     *
     * @param text text to convert
     * @return one byte per braille cell
     * @throws NullPointerException if {@code text} is {@code null}
     * @throws BraillifyException if the core cannot convert the input
     */
    public static byte[] encode(String text) {
        Objects.requireNonNull(text, "text must not be null");
        NativeLibraryLoader.ensureLoaded();
        return encodeNative(text);
    }

    /**
     * Converts text to Unicode braille characters (U+2800 through U+28FF).
     *
     * @param text text to convert
     * @return Unicode braille text
     * @throws NullPointerException if {@code text} is {@code null}
     * @throws BraillifyException if the core cannot convert the input
     */
    public static String translateToUnicode(String text) {
        Objects.requireNonNull(text, "text must not be null");
        NativeLibraryLoader.ensureLoaded();
        return translateToUnicodeNative(text);
    }

    /**
     * Converts text to the braille-font representation produced by the core engine.
     *
     * @param text text to convert
     * @return the core engine's braille-font representation
     * @throws NullPointerException if {@code text} is {@code null}
     * @throws BraillifyException if the core cannot convert the input
     */
    public static String translateToBrailleFont(String text) {
        Objects.requireNonNull(text, "text must not be null");
        NativeLibraryLoader.ensureLoaded();
        return translateToBrailleFontNative(text);
    }

    private static native byte[] encodeNative(String text);

    private static native String translateToUnicodeNative(String text);

    private static native String translateToBrailleFontNative(String text);

    // Only exported by debug native builds. Package-private for JUnit.
    static native void panicForTestingNative();
}
