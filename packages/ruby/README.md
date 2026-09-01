# braillify (Ruby)

한국어 텍스트를 한국 점자(2024 개정 한국 점자 규정)로 변환하는 라이브러리의 Ruby 바인딩입니다. Rust 네이티브 확장(magnus + rb-sys)으로 동작합니다.

## 설치

```bash
gem install braillify
```

주요 플랫폼(linux x86_64/arm64, macOS x86_64/arm64, Windows)용 프리컴파일 gem이 제공됩니다. 그 외 플랫폼은 소스 gem이 설치되며 Rust 툴체인이 필요합니다.

## 사용법

```ruby
require "braillify"

Braillify.translate_to_unicode("안녕하세요")      # => 점자 유니코드 String
Braillify.translate_to_braille_font("안녕하세요") # => 점자 폰트 String
Braillify.encode("안녕하세요")                    # => ASCII-8BIT String (점자 셀 바이트)
```

변환할 수 없는 입력은 `ArgumentError`를 발생시킵니다.

## 개발

```bash
bundle install
bundle exec rake compile   # 네이티브 확장 빌드
bundle exec rake test      # minitest 실행
```

Rust 쪽 단위 테스트는 저장소 루트에서 `cargo test -p braillify_rb`로 실행합니다 (Ruby 3.x 필요).

### Windows 개발 환경

RubyInstaller(ucrt) + DevKit(MSYS2) + **GNU Rust 툴체인**(`x86_64-pc-windows-gnu`) 조합을 사용합니다. bindgen이 libclang을 요구하므로 LLVM 설치 후 아래 환경변수가 필요합니다 (`<N>`은 LLVM 메이저 버전):

```powershell
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
$env:BINDGEN_EXTRA_CLANG_ARGS = '--target=x86_64-w64-mingw32 -I"C:/Program Files/LLVM/lib/clang/<N>/include" -I<Ruby설치경로>/msys64/ucrt64/include'
```

`rake compile`은 `ridk exec` (MSYS2 환경) 안에서 실행합니다.
