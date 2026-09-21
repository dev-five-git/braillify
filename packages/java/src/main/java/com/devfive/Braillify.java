/*
 * Copyright 2026-2026 the original author or authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
package com.devfive;

import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;

/**
 * The main entry point for the Braillify library.
 * This class handles the automatic loading of the platform-specific native library
 * and exposes the native encoding methods to Java.
 *
 * @author Yejeong Ham
 * @since 1.0.11
 */
public class Braillify {

	static {
		try {
			loadNative();
		} catch (Exception e) {
			throw new RuntimeException("Failed to load native library", e);
		}
	}

	private static void loadNative() throws Exception {
		String os = System.getProperty("os.name").toLowerCase();
		String arch = System.getProperty("os.arch").toLowerCase();
		String platform;
		String libName;

		if (os.contains("win")) {
			platform = "windows-x86_64";
			libName = "braillify_java.dll";
		} else if (os.contains("linux")) {
			platform = "linux-x86_64";
			libName = "libbraillify_java.so";
		} else if (os.contains("mac")) {
			if (arch.contains("aarch64") || arch.contains("arm")) {
				platform = "macos-aarch64";
			} else {
				platform = "macos-x86_64";
			}
			libName = "libbraillify_java.dylib";
		} else {
			throw new UnsupportedOperationException("Unsupported OS: " + os);
		}

		String resourcePath = "/natives/" + platform + "/" + libName;

		InputStream in = Braillify.class.getResourceAsStream(resourcePath);
		if (in == null) {
			throw new RuntimeException("Native library not found: " + resourcePath);
		}

		Path tempFile = Files.createTempFile("braillify-", libName);
		tempFile.toFile().deleteOnExit();

		Files.copy(in, tempFile, StandardCopyOption.REPLACE_EXISTING);
		in.close();

		System.load(tempFile.toAbsolutePath().toString());
	}

	public static native String encodeToUnicode(String input);
	public static native byte[] encode(String input);
}
