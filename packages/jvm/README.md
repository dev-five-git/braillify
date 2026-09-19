# braillify for Java and Kotlin

한국어 텍스트를 한국 점자로 변환하는 braillify의 JVM 바인딩입니다. Rust 코어를 JNI로 직접 호출하며 Java와 Kotlin에서 같은 JAR을 사용합니다.

## 설치

Maven Central 배포 후 Gradle에서는 다음과 같이 추가합니다.

```kotlin
dependencies {
    implementation("io.github.kdyann:braillify:BRAILLIFY_VERSION")
}
```

Maven:

```xml
<dependency>
  <groupId>io.github.kdyann</groupId>
  <artifactId>braillify</artifactId>
  <version>BRAILLIFY_VERSION</version>
</dependency>
```

## 사용법

Java:

```java
import io.github.kdyann.braillify.Braillify;

byte[] cells = Braillify.encode("안녕하세요");
String unicode = Braillify.translateToUnicode("안녕하세요");
String font = Braillify.translateToBrailleFont("안녕하세요");
```

Kotlin:

```kotlin
import io.github.kdyann.braillify.Braillify

val cells: ByteArray = Braillify.encode("안녕하세요")
val unicode: String = Braillify.translateToUnicode("안녕하세요")
val font: String = Braillify.translateToBrailleFont("안녕하세요")
```

`null`은 `NullPointerException`으로 거부합니다. 규정상 변환할 수 없는 문자, NUL 문자, 올바르지 않은 UTF-16 surrogate는 `BraillifyException`을 발생시킵니다. Rust panic이나 JNI/네이티브 로딩 실패는 `BraillifyInternalException`으로 구분합니다.

## 지원 환경

- Java 8 이상 (빌드 실행에는 JDK 17 이상 필요)
- Linux glibc x86_64 / aarch64
- macOS x86_64 / Apple Silicon
- Windows x86_64

Android, Windows arm64, Linux musl, GraalVM Native Image는 현재 지원하지 않습니다. Android는 NDK 바이너리와 AAR 패키징을 별도 작업으로 제공할 예정입니다.

## 로컬 빌드와 테스트

```bash
./gradlew test
./gradlew assembleJvmJar
```

`test`는 현재 호스트용 Rust JNI 라이브러리를 빌드하고 JAR 리소스로 넣은 뒤 실제 JVM 호출을 검사합니다. `assembleJvmJar` 결과는 `build/libs/braillify-<version>.jar`입니다.

크로스 컴파일한 release 라이브러리를 fat JAR에 넣을 때는 각 바이너리를 먼저 스테이징합니다.

```bash
./gradlew stageNativeLibrary \
  -PnativeTarget=linux-x86_64 \
  -PnativeLibrary=/absolute/path/libbraillify_jvm.so
./gradlew assembleJvmJar
```

지원하는 `nativeTarget` 값은 `linux-x86_64`, `linux-aarch64`, `macos-x86_64`, `macos-aarch64`, `windows-x86_64`입니다. 다섯 target을 같은 build 디렉터리에 순서대로 스테이징한 뒤 조립하면 단일 fat JAR에 모두 포함됩니다.

## 네이티브 로딩 문제 해결

JAR 안의 라이브러리는 class loader별 고유 임시 디렉터리에 콘텐츠 해시가 포함된 이름으로 추출됩니다. 임시 디렉터리가 `noexec`으로 마운트된 환경에서는 미리 설치한 라이브러리의 절대 경로를 지정할 수 있습니다.

```bash
java -Dbraillify.native.path=/opt/braillify/libbraillify_jvm.so ...
```

디렉터리를 지정하면 현재 플랫폼에 맞는 표준 파일명을 그 안에서 찾습니다. 최신 JDK가 native access 경고 또는 제한을 적용하는 경우 애플리케이션 실행 옵션에 `--enable-native-access=ALL-UNNAMED`를 추가하세요.

## 게시 산출물

`publishAllPublicationsToLocalStagingRepository`는 Maven 저장소 형태의 산출물을 `build/staging-repository`에 만듭니다. PGP 환경 변수를 설정한 뒤 `createCentralBundle`을 실행하면 `build/central/central-bundle.zip`이 생성됩니다.

- `MAVEN_SIGNING_KEY`
- `MAVEN_SIGNING_PASSWORD`
- `MAVEN_CENTRAL_USERNAME`
- `MAVEN_CENTRAL_PASSWORD`

`publishToCentralPortal`은 bundle을 Central Portal에 올리고 검증 결과가 나올 때까지 기다립니다. 같은 이름의 Gradle property도 환경 변수 대신 사용할 수 있습니다.
