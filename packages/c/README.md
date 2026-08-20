# braillify-c

한국어 텍스트를 2024 개정 한국 점자 규정에 따라 변환하는 Braillify의 C 바인딩입니다. 동일한 헤더를 C와 C++에서 사용할 수 있습니다.

## 빌드

저장소 루트에서 동적·정적 라이브러리를 빌드합니다.

```bash
cargo build --release -p braillify-c
```

빌드 결과는 플랫폼에 따라 `target/release/libbraillify_c.so`, `libbraillify_c.dylib`, `braillify_c.dll` 및 정적 라이브러리로 생성됩니다. 공개 헤더는 `include/braillify.h`입니다.

## 사용법

```c
#include <braillify.h>
#include <stdio.h>

int main(void) {
    char *result = braillify_encode_unicode("안녕하세요");
    if (result == NULL) {
        char *error = braillify_last_error();
        fprintf(stderr, "%s\n", error != NULL ? error : "unknown error");
        braillify_string_free(error);
        return 1;
    }

    puts(result);
    braillify_string_free(result);
    return 0;
}
```

Linux에서는 다음과 같이 링크할 수 있습니다.

```bash
cc example.c -Ipath/to/packages/c/include -Lpath/to/target/release \
  -lbraillify_c -o example
```

동적 라이브러리를 실행 시 검색할 수 있도록 `LD_LIBRARY_PATH`(Linux), `DYLD_LIBRARY_PATH`(macOS) 또는 `PATH`(Windows)를 설정하거나 애플리케이션에 rpath를 지정해야 합니다.

## API와 메모리 소유권

- 입력은 NUL로 끝나는 UTF-8 문자열이어야 합니다.
- `braillify_encode_unicode`와 `braillify_encode_braille_font`의 반환값은 `braillify_string_free`로 해제합니다.
- `braillify_encode`의 반환값은 함께 받은 길이를 그대로 사용해 `braillify_bytes_free`로 해제합니다.
- 실패 시 인코딩 함수는 `NULL`을 반환합니다. `braillify_last_error`가 반환한 메시지도 `braillify_string_free`로 해제합니다.
- 마지막 오류는 스레드별로 저장되며, 다음 인코딩 호출이 시작되면 초기화됩니다.
- 모든 해제 함수는 `NULL`을 허용합니다.

## C++

헤더가 선언을 `extern "C"`로 감싸므로 C++에서도 그대로 포함할 수 있습니다. 반환된 메모리는 `delete`나 `free`가 아니라 반드시 위의 Braillify 해제 함수로 반환해야 합니다.

## 테스트

```bash
cargo test -p braillify-c
make -C packages/c test
bun run test:c
```

`make` 명령은 실제 C11 컴파일러로 공개 헤더와 동적 라이브러리를 링크해 스모크 테스트를 실행합니다. `bun run test:c`는 같은 공개 헤더를 C11과 C++17로 각각 컴파일하고 라이브러리에 링크한 뒤 두 실행 파일을 모두 검증합니다.
