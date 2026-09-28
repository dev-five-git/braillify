# Braillify .NET

Rust 기반 크로스플랫폼 한국어 점역 라이브러리 .NET 바인딩

## 패키지

| 패키지         | 설명       | NuGet                                                                                                |
| -------------- | ---------- | ---------------------------------------------------------------------------------------------------- |
| `BraillifyNet` | 라이브러리 | [![NuGet](https://img.shields.io/nuget/v/BraillifyNet)](https://www.nuget.org/packages/BraillifyNet) |
| `Braillify`    | CLI 도구   | [![NuGet](https://img.shields.io/nuget/v/Braillify)](https://www.nuget.org/packages/Braillify)       |

## 라이브러리 설치

```bash
dotnet add package BraillifyNet
```

## 라이브러리 사용법

```csharp
using Braillify;

// 텍스트를 점자 유니코드로 변환
string braille = Braillify.EncodeToUnicode("안녕하세요");
// 결과: "⠣⠒⠉⠻⠚⠠⠝⠬"

// 텍스트를 점자 바이트 배열로 변환
byte[] bytes = Braillify.Encode("안녕하세요");

// 텍스트를 점자 폰트 문자열로 변환
string font = Braillify.EncodeToBrailleFont("안녕하세요");

// 묵자 모양만으로 규정을 정할 수 없는 글은 문맥("science", "math", "korean" 등)을 밝힌다
string science = Braillify.EncodeToUnicode("pOH", "science");
// 결과: "⠴⠏⠠⠕⠠⠓"
```

## CLI 설치

```bash
# 글로벌 설치
dotnet tool install -g Braillify

# 설치 없이 실행 (.NET 10+)
dnx braillify "안녕하세요"
```

## CLI 사용법

```bash
# 텍스트 변환
braillify "안녕하세요"
# 출력: ⠣⠒⠉⠻⠚⠠⠝⠬

# 파이프 입력
echo "안녕하세요" | braillify

# REPL 모드
braillify
```

## 지원 플랫폼

- Windows (x64, x86, arm64)
- Linux (x64, arm64)
- macOS (x64, arm64)

## 지원 .NET 버전

- .NET Standard 2.0
- .NET Core 3.1
- .NET 5.0, 6.0, 7.0, 8.0, 9.0, 10.0

## 라이선스

Apache-2.0

## 링크

- [GitHub](https://github.com/dev-five-git/braillify)
- [홈페이지](https://braillify.kr)
