package io.github.kdyann.braillify;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.InputStream;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.nio.file.StandardCopyOption;
import java.util.jar.JarFile;
import java.util.stream.Stream;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.Arguments;
import org.junit.jupiter.params.provider.MethodSource;

class NativeLibraryLoaderTest {
    static Stream<Arguments> supportedPlatforms() {
        return Stream.of(
                Arguments.of("Linux", "amd64", "linux-x86_64"),
                Arguments.of("Linux", "aarch64", "linux-aarch64"),
                Arguments.of("Mac OS X", "x86_64", "macos-x86_64"),
                Arguments.of("Darwin", "arm64", "macos-aarch64"),
                Arguments.of("Windows 11", "amd64", "windows-x86_64"));
    }

    @ParameterizedTest
    @MethodSource("supportedPlatforms")
    void normalizesSupportedPlatforms(String os, String arch, String expected) {
        assertEquals(expected, NativeLibraryLoader.platform(os, arch, "OpenJDK Runtime", "OpenJDK VM"));
    }

    @Test
    void detectsAndroidBeforeLinux() {
        UnsupportedOperationException error = assertThrows(UnsupportedOperationException.class,
                () -> NativeLibraryLoader.platform("Linux", "aarch64", "Android Runtime", "Dalvik"));
        assertTrue(error.getMessage().contains("Android"));
    }

    @Test
    void rejectsUnsupportedArchitectureWithObservedValues() {
        UnsupportedOperationException error = assertThrows(UnsupportedOperationException.class,
                () -> NativeLibraryLoader.platform("Linux", "riscv64", "OpenJDK", "OpenJDK"));
        assertTrue(error.getMessage().contains("Linux"));
        assertTrue(error.getMessage().contains("riscv64"));
    }

    @Test
    void mapsLibraryNames() {
        assertEquals("libbraillify_jvm.so", NativeLibraryLoader.libraryFileName("linux-x86_64"));
        assertEquals("libbraillify_jvm.dylib", NativeLibraryLoader.libraryFileName("macos-aarch64"));
        assertEquals("braillify_jvm.dll", NativeLibraryLoader.libraryFileName("windows-x86_64"));
    }

    @Test
    void loadingIsIdempotent() {
        NativeLibraryLoader.ensureLoaded();
        NativeLibraryLoader.ensureLoaded();
    }

    @Test
    void isolatedClassLoadersCanEachCallTheFatJar() throws Exception {
        URL jar = Paths.get(System.getProperty("braillify.test.jar")).toUri().toURL();
        String first = invokeFromIsolatedLoader(jar);
        String second = invokeFromIsolatedLoader(jar);
        assertEquals(first, second);
        assertTrue(first.codePoints().allMatch(value -> value >= 0x2800 && value <= 0x28ff));
    }

    @Test
    void loadsNativeLibraryFromFileOverride() throws Exception {
        Path nativeLibrary = extractHostNativeLibrary("file-override");
        String result = invokeWithNativeOverride(nativeLibrary);
        assertTrue(result.codePoints().allMatch(value -> value >= 0x2800 && value <= 0x28ff));
    }

    @Test
    void loadsNativeLibraryFromDirectoryOverride() throws Exception {
        Path nativeLibrary = extractHostNativeLibrary("directory-override");
        String result = invokeWithNativeOverride(nativeLibrary.getParent());
        assertTrue(result.codePoints().allMatch(value -> value >= 0x2800 && value <= 0x28ff));
    }

    @Test
    void wrapsMissingNativeOverrideWithHelpfulException() throws Exception {
        Path missing = Files.createTempDirectory("braillify-missing-").resolve("missing-native-library");
        InvocationTargetException error = assertThrows(InvocationTargetException.class,
                () -> invokeWithNativeOverride(missing));
        Throwable cause = error.getCause();
        assertEquals("io.github.kdyann.braillify.BraillifyInternalException", cause.getClass().getName());
        assertTrue(cause.getMessage().contains("braillify.native.path"));
        assertTrue(cause.getMessage().contains(missing.toAbsolutePath().toString()));
    }

    private static String invokeFromIsolatedLoader(URL jar) throws Exception {
        try (URLClassLoader loader = new URLClassLoader(new URL[] {jar}, null)) {
            Class<?> api = Class.forName("io.github.kdyann.braillify.Braillify", true, loader);
            Method method = api.getMethod("translateToUnicode", String.class);
            return (String) method.invoke(null, "안녕");
        }
    }

    private static String invokeWithNativeOverride(Path override) throws Exception {
        String property = "braillify.native.path";
        String previous = System.getProperty(property);
        System.setProperty(property, override.toAbsolutePath().toString());
        try {
            URL jar = Paths.get(System.getProperty("braillify.test.jar")).toUri().toURL();
            return invokeFromIsolatedLoader(jar);
        } finally {
            if (previous == null) {
                System.clearProperty(property);
            } else {
                System.setProperty(property, previous);
            }
        }
    }

    private static Path extractHostNativeLibrary(String prefix) throws Exception {
        Path jarPath = Paths.get(System.getProperty("braillify.test.jar"));
        String platform = NativeLibraryLoader.currentPlatform();
        String fileName = NativeLibraryLoader.libraryFileName(platform);
        String resource = "META-INF/native/" + platform + "/" + fileName;
        Path directory = Files.createTempDirectory("braillify-" + prefix + "-");
        Path destination = directory.resolve(fileName);
        try (JarFile jar = new JarFile(jarPath.toFile())) {
            if (jar.getJarEntry(resource) == null) {
                throw new AssertionError("Missing test native resource: " + resource);
            }
            try (InputStream input = jar.getInputStream(jar.getJarEntry(resource))) {
                Files.copy(input, destination, StandardCopyOption.REPLACE_EXISTING);
            }
        }
        return destination;
    }
}
