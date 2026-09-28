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

    /**
     * Converts text read in a named context to braille cell values. The context
     * ({@code "science"}, {@code "math"}, {@code "korean"}, ...) decides input
     * whose print shape alone does not, such as {@code 44+XX} in science.
     *
     * @param text text to convert
     * @param context context name
     * @return one byte per braille cell
     * @throws NullPointerException if {@code text} or {@code context} is {@code null}
     * @throws BraillifyException if the context is unknown or the core cannot convert the input
     */
    public static byte[] encode(String text, String context) {
        Objects.requireNonNull(text, "text must not be null");
        Objects.requireNonNull(context, "context must not be null");
        NativeLibraryLoader.ensureLoaded();
        return encodeInContextNative(text, context);
    }

    /**
     * Converts text read in a named context to Unicode braille characters.
     *
     * @param text text to convert
     * @param context context name, as for {@link #encode(String, String)}
     * @return Unicode braille text
     * @throws NullPointerException if {@code text} or {@code context} is {@code null}
     * @throws BraillifyException if the context is unknown or the core cannot convert the input
     */
    public static String translateToUnicode(String text, String context) {
        Objects.requireNonNull(text, "text must not be null");
        Objects.requireNonNull(context, "context must not be null");
        NativeLibraryLoader.ensureLoaded();
        return translateToUnicodeInContextNative(text, context);
    }

    /**
     * Converts text read in a named context to the braille-font representation.
     *
     * @param text text to convert
     * @param context context name, as for {@link #encode(String, String)}
     * @return the core engine's braille-font representation
     * @throws NullPointerException if {@code text} or {@code context} is {@code null}
     * @throws BraillifyException if the context is unknown or the core cannot convert the input
     */
    public static String translateToBrailleFont(String text, String context) {
        Objects.requireNonNull(text, "text must not be null");
        Objects.requireNonNull(context, "context must not be null");
        NativeLibraryLoader.ensureLoaded();
        return translateToBrailleFontInContextNative(text, context);
    }

    private static native byte[] encodeNative(String text);

    private static native String translateToUnicodeNative(String text);

    private static native String translateToBrailleFontNative(String text);

    private static native byte[] encodeInContextNative(String text, String context);

    private static native String translateToUnicodeInContextNative(String text, String context);

    private static native String translateToBrailleFontInContextNative(String text, String context);

    // Only exported by debug native builds. Package-private for JUnit.
    static native void panicForTestingNative();
}
