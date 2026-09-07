package io.github.kdyann.braillify;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.AtomicMoveNotSupportedException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.nio.file.StandardCopyOption;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.Locale;

final class NativeLibraryLoader {
    private static final Object LOAD_LOCK = new Object();
    private static volatile boolean loaded;

    private NativeLibraryLoader() {
    }

    static void ensureLoaded() {
        if (loaded) {
            return;
        }

        synchronized (LOAD_LOCK) {
            if (loaded) {
                return;
            }
            loadNativeLibrary();
            loaded = true;
        }
    }

    private static void loadNativeLibrary() {
        String platform = currentPlatform();
        String fileName = libraryFileName(platform);
        String override = System.getProperty("braillify.native.path");
        if (override != null && !override.trim().isEmpty()) {
            Path path = Paths.get(override).toAbsolutePath();
            if (Files.isDirectory(path)) {
                path = path.resolve(fileName);
            }
            loadAbsolute(path, "system property braillify.native.path");
            return;
        }

        String resource = "/META-INF/native/" + platform + "/" + fileName;
        try (InputStream stream = NativeLibraryLoader.class.getResourceAsStream(resource)) {
            if (stream == null) {
                throw unavailable("Native resource is missing: " + resource, platform, null);
            }
            byte[] bytes = readAllBytes(stream);
            String digest = sha256(bytes).substring(0, 16);
            Path directory = Files.createTempDirectory("braillify-" + digest + "-");
            directory.toFile().deleteOnExit();
            Path destination = directory.resolve(withDigest(fileName, digest));
            Path temporary = Files.createTempFile(directory, "extract-", ".tmp");
            temporary.toFile().deleteOnExit();
            Files.write(temporary, bytes);
            moveAtomically(temporary, destination);
            destination.toFile().deleteOnExit();
            loadAbsolute(destination, "JAR resource " + resource);
        } catch (IOException error) {
            throw unavailable("Could not extract native resource " + resource, platform, error);
        }
    }

    static String currentPlatform() {
        return platform(System.getProperty("os.name"), System.getProperty("os.arch"),
                System.getProperty("java.runtime.name"), System.getProperty("java.vm.name"));
    }

    static String platform(String osName, String osArch, String runtimeName, String vmName) {
        String os = normalize(osName);
        String arch = normalize(osArch);
        String runtime = normalize(runtimeName) + " " + normalize(vmName);

        if (runtime.contains("android") || runtime.contains("dalvik")) {
            throw new UnsupportedOperationException(
                    "Android is not supported by the JVM JAR; use the future Android AAR binding");
        }

        String normalizedOs;
        if (os.contains("linux")) {
            normalizedOs = "linux";
        } else if (os.contains("mac") || os.contains("darwin")) {
            normalizedOs = "macos";
        } else if (os.contains("windows")) {
            normalizedOs = "windows";
        } else {
            throw unsupported(osName, osArch);
        }

        String normalizedArch;
        if (arch.equals("x8664") || arch.equals("amd64")) {
            normalizedArch = "x86_64";
        } else if (arch.equals("aarch64") || arch.equals("arm64")) {
            normalizedArch = "aarch64";
        } else {
            throw unsupported(osName, osArch);
        }

        if (normalizedOs.equals("windows") && normalizedArch.equals("aarch64")) {
            throw unsupported(osName, osArch);
        }
        return normalizedOs + "-" + normalizedArch;
    }

    static String libraryFileName(String platform) {
        if (platform.startsWith("windows-")) {
            return "braillify_jvm.dll";
        }
        if (platform.startsWith("macos-")) {
            return "libbraillify_jvm.dylib";
        }
        return "libbraillify_jvm.so";
    }

    private static String normalize(String value) {
        return value == null ? "" : value.toLowerCase(Locale.ROOT).replaceAll("[^a-z0-9]", "");
    }

    private static UnsupportedOperationException unsupported(String os, String arch) {
        return new UnsupportedOperationException(
                "Unsupported braillify JVM platform: os.name=" + os + ", os.arch=" + arch);
    }

    private static void loadAbsolute(Path path, String source) {
        if (!Files.isRegularFile(path)) {
            throw unavailable("Native library does not exist at " + path + " (from " + source + ")",
                    safeCurrentPlatform(), null);
        }
        try {
            System.load(path.toString());
        } catch (UnsatisfiedLinkError error) {
            throw unavailable("Could not load native library " + path + " (from " + source + ")",
                    safeCurrentPlatform(), error);
        }
    }

    private static BraillifyInternalException unavailable(String message, String platform, Throwable cause) {
        String detail = message + ". Detected platform: " + platform
                + ". If temporary directories are mounted noexec, set -Dbraillify.native.path=/absolute/path/to/library";
        return cause == null ? new BraillifyInternalException(detail)
                : new BraillifyInternalException(detail, cause);
    }

    private static String safeCurrentPlatform() {
        try {
            return currentPlatform();
        } catch (RuntimeException ignored) {
            return System.getProperty("os.name") + "-" + System.getProperty("os.arch");
        }
    }

    private static byte[] readAllBytes(InputStream input) throws IOException {
        java.io.ByteArrayOutputStream output = new java.io.ByteArrayOutputStream();
        byte[] buffer = new byte[8192];
        int read;
        while ((read = input.read(buffer)) != -1) {
            output.write(buffer, 0, read);
        }
        return output.toByteArray();
    }

    private static String sha256(byte[] bytes) {
        try {
            byte[] digest = MessageDigest.getInstance("SHA-256").digest(bytes);
            StringBuilder hex = new StringBuilder(digest.length * 2);
            for (byte value : digest) {
                hex.append(String.format(Locale.ROOT, "%02x", value & 0xff));
            }
            return hex.toString();
        } catch (NoSuchAlgorithmException impossible) {
            throw new AssertionError("Every Java runtime must provide SHA-256", impossible);
        }
    }

    private static String withDigest(String fileName, String digest) {
        int extension = fileName.lastIndexOf('.');
        return extension < 0 ? fileName + "-" + digest
                : fileName.substring(0, extension) + "-" + digest + fileName.substring(extension);
    }

    private static void moveAtomically(Path source, Path destination) throws IOException {
        try {
            Files.move(source, destination, StandardCopyOption.ATOMIC_MOVE);
        } catch (AtomicMoveNotSupportedException ignored) {
            Files.move(source, destination, StandardCopyOption.REPLACE_EXISTING);
        }
    }
}
