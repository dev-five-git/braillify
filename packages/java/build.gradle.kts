import org.gradle.api.GradleException
import org.gradle.api.file.DuplicatesStrategy
import org.gradle.api.tasks.bundling.Jar
import org.gradle.api.tasks.compile.JavaCompile

plugins {
    id("java")
    id("maven-publish")
    application
}

data class NativeConfig(
    val rustTarget: String,
    val nativeLibName: String,
    val nativePath: String
)

val osName = System.getProperty("os.name").lowercase()
val osArch = System.getProperty("os.arch").lowercase()

val nativeConfig: NativeConfig = when {
    osName.contains("win") -> NativeConfig(
        rustTarget = "x86_64-pc-windows-msvc",
        nativeLibName = "braillify_java.dll",
        nativePath = "windows-x86_64"
    )

    osName.contains("nix") || osName.contains("nux") -> NativeConfig(
        rustTarget = "x86_64-unknown-linux-gnu",
        nativeLibName = "libbraillify_java.so",
        nativePath = "linux-x86_64"
    )

    osName.contains("mac") && (osArch == "aarch64" || osArch == "arm64") -> NativeConfig(
        rustTarget = "aarch64-apple-darwin",
        nativeLibName = "libbraillify_java.dylib",
        nativePath = "darwin-aarch64"
    )

    osName.contains("mac") -> NativeConfig(
        rustTarget = "x86_64-apple-darwin",
        nativeLibName = "libbraillify_java.dylib",
        nativePath = "darwin-x86_64"
    )

    else -> throw GradleException("Unsupported OS for cargo build: $osName ($osArch)")
}

group = "com.devfive"
version = project.findProperty("releaseVersion") ?: "1.0.11"

application {
    mainClass.set("com.devfive.Braillify")
}

repositories {
    mavenCentral()
}

dependencies {
    implementation("org.scijava:native-lib-loader:2.5.0")
    testImplementation(platform("org.junit:junit-bom:5.10.0"))
    testImplementation("org.junit.jupiter:junit-jupiter")
}

tasks.test {
    useJUnitPlatform()
}

tasks.withType<Jar> {
    duplicatesStrategy = DuplicatesStrategy.EXCLUDE
}

val rustProjectDir = file("src/main/java/com/devfive")
val cargoTargetDir = layout.buildDirectory.dir("cargo-target").get().asFile

tasks.register<Exec>("cargoBuild") {
    workingDir(rustProjectDir)
    environment("CARGO_TARGET_DIR", cargoTargetDir.absolutePath)

    commandLine(
        "cargo",
        "build",
        "--release",
        "--target",
        nativeConfig.rustTarget
    )
}

tasks.register<Copy>("copyNativeLib") {
    dependsOn("cargoBuild")
    from("${cargoTargetDir}/${nativeConfig.rustTarget}/release/${nativeConfig.nativeLibName}")
    into("src/main/resources/natives/${nativeConfig.nativePath}")
}


tasks.named("processResources") {
    dependsOn("copyNativeLib")
}

tasks.named<Jar>("jar") {
    manifest {
        attributes["Main-Class"] = "com.devfive.Braillify"
    }

    from(sourceSets.main.get().output)
    from(configurations.runtimeClasspath.get().map { if (it.isDirectory) it else zipTree(it) })
}

tasks.register<Jar>("deployJar") {
    archiveBaseName.set(project.name)

    manifest {
        attributes["Main-Class"] = "com.devfive.Braillify"
    }

    val compileJava = tasks.named<JavaCompile>("compileJava")
    dependsOn(compileJava)
    from(compileJava.map { it.destinationDirectory })
    from("src/main/resources")
    from(configurations.runtimeClasspath.get().map { if (it.isDirectory) it else zipTree(it) })
}

val prebuiltJar = layout.projectDirectory.file("jar/${project.name}-${project.version}.jar")

publishing {
    publications {
        create<MavenPublication>("mavenPrebuilt") {
            groupId = project.group.toString()
            artifactId = project.name
            version = project.version.toString()

            artifact(prebuiltJar.asFile) {
                extension = "jar"
            }

            pom {
                name.set(project.name)
                licenses {
                    license {
                        name.set("Apache License 2.0")
                        url.set("https://www.apache.org/licenses/LICENSE-2.0")
                    }
                }
            }
        }
    }
}