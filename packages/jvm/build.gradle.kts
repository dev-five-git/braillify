import java.net.HttpURLConnection
import java.net.URL
import java.nio.charset.StandardCharsets
import java.security.MessageDigest
import java.util.Base64
import java.util.Locale
import java.util.zip.ZipEntry
import java.util.zip.ZipOutputStream

plugins {
    `java-library`
    `maven-publish`
    signing
}

group = "io.github.kdyann"
version = "0.1.0"
description = "Korean text-to-braille conversion for Java and Kotlin"

repositories {
    mavenCentral()
}

java {
    withSourcesJar()
    withJavadocJar()
}

tasks.withType<JavaCompile>().configureEach {
    options.release.set(8)
    options.encoding = "UTF-8"
}

dependencies {
    testImplementation(platform("org.junit:junit-bom:5.13.4"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

val generatedNativeResources = layout.buildDirectory.dir("generated/native-resources")
val externalStageMarkers = layout.buildDirectory.dir("staged-native-markers")
sourceSets.main {
    resources.srcDir(generatedNativeResources)
}

fun normalizedHostPlatform(): String {
    val os = System.getProperty("os.name").lowercase(Locale.ROOT)
    val arch = System.getProperty("os.arch").lowercase(Locale.ROOT).replace(Regex("[^a-z0-9]"), "")
    val normalizedOs = when {
        os.contains("linux") -> "linux"
        os.contains("mac") || os.contains("darwin") -> "macos"
        os.contains("windows") -> "windows"
        else -> throw GradleException("Unsupported JVM native build OS: ${System.getProperty("os.name")}")
    }
    val normalizedArch = when (arch) {
        "x8664", "amd64" -> "x86_64"
        "aarch64", "arm64" -> "aarch64"
        else -> throw GradleException("Unsupported JVM native build architecture: ${System.getProperty("os.arch")}")
    }
    if (normalizedOs == "windows" && normalizedArch == "aarch64") {
        throw GradleException("Windows aarch64 is not supported in the first JVM release")
    }
    return "$normalizedOs-$normalizedArch"
}

fun libraryFileName(platform: String): String = when {
    platform.startsWith("windows-") -> "braillify_jvm.dll"
    platform.startsWith("macos-") -> "libbraillify_jvm.dylib"
    else -> "libbraillify_jvm.so"
}

val hostPlatform = normalizedHostPlatform()
val rustTargetDirectory = layout.buildDirectory.dir("rust-target")
val hostNativeLibrary = rustTargetDirectory.map {
    it.file("debug/${libraryFileName(hostPlatform)}")
}

val buildRustNative by tasks.registering(Exec::class) {
    group = "build"
    description = "Builds the host JNI library with Cargo."
    workingDir = rootProject.projectDir
    commandLine("cargo", "build", "-p", "braillify-jvm")
    environment("CARGO_TARGET_DIR", rustTargetDirectory.get().asFile.absolutePath)
    inputs.files(
        fileTree("src") { include("**/*.rs") },
        file("Cargo.toml"),
        file("../../Cargo.toml"),
        file("../../Cargo.lock"),
        file("../../libs/braillify"),
    )
    outputs.file(hostNativeLibrary)
    onlyIf {
        !externalStageMarkers.get().file(hostPlatform).asFile.exists()
    }
}

val prepareNativeResources by tasks.registering(Copy::class) {
    group = "build"
    description = "Copies the host JNI library into its JAR resource path."
    dependsOn(buildRustNative)
    from(hostNativeLibrary)
    into(generatedNativeResources.map { it.dir("META-INF/native/$hostPlatform") })
    // Cross-target release staging writes this marker. Never replace a staged
    // release library with the host debug build while assembling the fat JAR.
    onlyIf {
        !externalStageMarkers.get()
            .file(hostPlatform)
            .asFile.exists()
    }
}

val stageNativeLibrary by tasks.registering {
    group = "build"
    description = "Stages a cross-compiled JNI library (-PnativeTarget and -PnativeLibrary)."
    val target = providers.gradleProperty("nativeTarget")
    val library = providers.gradleProperty("nativeLibrary")
    inputs.property("nativeTarget", target)
    inputs.file(library)
    outputs.dir(generatedNativeResources)
    doLast {
        val targetValue = target.orNull
            ?: throw GradleException("stageNativeLibrary requires -PnativeTarget")
        val supported = setOf(
            "linux-x86_64", "linux-aarch64", "macos-x86_64", "macos-aarch64", "windows-x86_64"
        )
        if (targetValue !in supported) {
            throw GradleException("Unsupported nativeTarget: $targetValue")
        }
        val source = library.orNull?.let(::file)
            ?: throw GradleException("stageNativeLibrary requires -PnativeLibrary")
        if (!source.isFile) {
            throw GradleException("Native library does not exist: $source")
        }
        copy {
            from(source)
            into(generatedNativeResources.get().dir("META-INF/native/$targetValue"))
            rename { libraryFileName(targetValue) }
        }
        externalStageMarkers.get()
            .file(targetValue)
            .asFile
            .apply {
                parentFile.mkdirs()
                writeText("release\n")
            }
    }
}

tasks.processResources {
    dependsOn(prepareNativeResources)
}

tasks.test {
    dependsOn(prepareNativeResources, tasks.jar)
    useJUnitPlatform()
    systemProperty("java.library.path", "")
    systemProperty("braillify.test.jar", tasks.jar.get().archiveFile.get().asFile.absolutePath)
}

tasks.jar {
    archiveBaseName.set("braillify")
    manifest {
        attributes(
            "Automatic-Module-Name" to "io.github.kdyann.braillify",
            "Implementation-Title" to "braillify",
            "Implementation-Version" to project.version,
        )
    }
    duplicatesStrategy = DuplicatesStrategy.FAIL
}

tasks.named<Jar>("sourcesJar") {
    dependsOn(prepareNativeResources)
    exclude("META-INF/native/**")
}

val assembleJvmJar by tasks.registering {
    group = "build"
    description = "Assembles the JVM fat JAR with all currently staged native libraries."
    dependsOn(tasks.jar)
}

publishing {
    publications {
        create<MavenPublication>("mavenJava") {
            from(components["java"])
            artifactId = "braillify"
            pom {
                name.set("braillify")
                description.set(project.description)
                url.set("https://braillify.kr")
                licenses {
                    license {
                        name.set("The Apache License, Version 2.0")
                        url.set("https://www.apache.org/licenses/LICENSE-2.0.txt")
                        distribution.set("repo")
                    }
                }
                developers {
                    developer {
                        id.set("kdyann")
                        name.set("JeongMin Oh")
                        email.set("owjs39@gmail.com")
                    }
                }
                scm {
                    connection.set("scm:git:https://github.com/dev-five-git/braillify.git")
                    developerConnection.set("scm:git:ssh://git@github.com/dev-five-git/braillify.git")
                    url.set("https://github.com/dev-five-git/braillify")
                }
            }
        }
    }
    repositories {
        maven {
            name = "localStaging"
            url = uri(layout.buildDirectory.dir("staging-repository"))
        }
    }
}

fun secret(name: String): String? = providers.gradleProperty(name).orNull ?: System.getenv(name)
val signingKey = secret("MAVEN_SIGNING_KEY")
val signingPassword = secret("MAVEN_SIGNING_PASSWORD")
signing {
    if (!signingKey.isNullOrBlank()) {
        useInMemoryPgpKeys(signingKey, signingPassword)
        sign(publishing.publications["mavenJava"])
    }
}

fun digest(file: File, algorithm: String): String {
    val messageDigest = MessageDigest.getInstance(algorithm)
    file.inputStream().buffered().use { input ->
        val buffer = ByteArray(8192)
        while (true) {
            val count = input.read(buffer)
            if (count < 0) break
            messageDigest.update(buffer, 0, count)
        }
    }
    return messageDigest.digest().joinToString("") { "%02x".format(it) }
}

val createCentralBundle by tasks.registering {
    group = "publishing"
    description = "Creates the Maven Central Portal deployment bundle."
    dependsOn("publishAllPublicationsToLocalStagingRepository")
    val repository = layout.buildDirectory.dir("staging-repository")
    val output = layout.buildDirectory.file("central/central-bundle.zip")
    inputs.dir(repository)
    outputs.file(output)
    doLast {
        if (signingKey.isNullOrBlank()) {
            throw GradleException("MAVEN_SIGNING_KEY is required to create a Central bundle")
        }
        val root = repository.get().asFile
        val versionDirectory = root.resolve("io/github/kdyann/braillify/${project.version}")
        if (!versionDirectory.isDirectory) {
            throw GradleException("Maven publication directory is missing: $versionDirectory")
        }
        val originals = versionDirectory.walkTopDown()
            .filter { it.isFile && !it.extension.matches(Regex("asc|md5|sha1|sha256|sha512")) }
            .toList()
        originals.forEach { source ->
            mapOf("md5" to "MD5", "sha1" to "SHA-1", "sha256" to "SHA-256", "sha512" to "SHA-512")
                .forEach { (extension, algorithm) ->
                    File(source.parentFile, "${source.name}.$extension").writeText(digest(source, algorithm))
                }
        }
        val bundle = output.get().asFile
        bundle.parentFile.mkdirs()
        ZipOutputStream(bundle.outputStream().buffered()).use { zip ->
            versionDirectory.walkTopDown()
                .filter(File::isFile)
                .sortedBy { it.relativeTo(root).invariantSeparatorsPath }
                .forEach { source ->
                zip.putNextEntry(ZipEntry(source.relativeTo(root).invariantSeparatorsPath))
                source.inputStream().use { it.copyTo(zip) }
                zip.closeEntry()
            }
        }
    }
}

val publishToCentralPortal by tasks.registering {
    group = "publishing"
    description = "Uploads the bundle to Central Portal and waits for validation."
    dependsOn(createCentralBundle)
    doLast {
        val username = secret("MAVEN_CENTRAL_USERNAME")
            ?: throw GradleException("MAVEN_CENTRAL_USERNAME is required")
        val password = secret("MAVEN_CENTRAL_PASSWORD")
            ?: throw GradleException("MAVEN_CENTRAL_PASSWORD is required")
        val token = Base64.getEncoder().encodeToString("$username:$password".toByteArray(StandardCharsets.UTF_8))
        val bundle = layout.buildDirectory.file("central/central-bundle.zip").get().asFile
        val boundary = "BraillifyBoundary${System.nanoTime()}"
        val endpoint = URL("https://central.sonatype.com/api/v1/publisher/upload?publishingType=AUTOMATIC&name=braillify-${project.version}")
        val connection = endpoint.openConnection() as HttpURLConnection
        connection.requestMethod = "POST"
        connection.connectTimeout = 30_000
        connection.readTimeout = 120_000
        connection.setRequestProperty("Authorization", "Bearer $token")
        connection.setRequestProperty("Content-Type", "multipart/form-data; boundary=$boundary")
        connection.doOutput = true
        connection.outputStream.use { outputStream ->
            outputStream.write("--$boundary\r\nContent-Disposition: form-data; name=\"bundle\"; filename=\"central-bundle.zip\"\r\nContent-Type: application/octet-stream\r\n\r\n".toByteArray(StandardCharsets.UTF_8))
            bundle.inputStream().use { it.copyTo(outputStream) }
            outputStream.write("\r\n--$boundary--\r\n".toByteArray(StandardCharsets.UTF_8))
        }
        val responseCode = connection.responseCode
        val response = (if (responseCode < 400) connection.inputStream else connection.errorStream)
            ?.bufferedReader()?.use { it.readText() }.orEmpty()
        if (responseCode !in 200..299) {
            throw GradleException("Central Portal upload failed ($responseCode): $response")
        }
        val deploymentId = response.trim().trim('"')
        logger.lifecycle("Central Portal accepted deployment $deploymentId")

        repeat(120) {
            Thread.sleep(5_000)
            val statusConnection = URL("https://central.sonatype.com/api/v1/publisher/status?id=$deploymentId")
                .openConnection() as HttpURLConnection
            statusConnection.requestMethod = "POST"
            statusConnection.connectTimeout = 30_000
            statusConnection.readTimeout = 30_000
            statusConnection.setRequestProperty("Authorization", "Bearer $token")
            val statusCode = statusConnection.responseCode
            val statusBody = (if (statusCode < 400) statusConnection.inputStream else statusConnection.errorStream)
                ?.bufferedReader()?.use { it.readText() }.orEmpty()
            if (statusCode !in 200..299) {
                throw GradleException("Central Portal status failed ($statusCode): $statusBody")
            }
            logger.lifecycle("Central Portal status: $statusBody")
            when {
                statusBody.contains("PUBLISHED") -> return@doLast
                statusBody.contains("FAILED") -> throw GradleException("Central Portal validation failed: $statusBody")
            }
        }
        throw GradleException("Timed out waiting for Central Portal validation")
    }
}
