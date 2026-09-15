# Braillify App

Tauri 2 + Next.js(App Router) 기반 Braillify 앱. 하나의 코드베이스로
데스크톱(Windows·macOS)과 모바일(iOS·Android)을 대상으로 합니다.
점역 엔진은 workspace 패키지 `braillify`(WASM)를 웹뷰에서 직접 호출하며,
Tauri IPC 커맨드를 두지 않아 모든 플랫폼이 동일한 경로를 사용합니다.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## 모바일 프로젝트 준비 (clone 후 최초 1회)

`src-tauri/gen/`(Xcode·Android 네이티브 프로젝트)은 전부 생성물이라 저장소에
커밋하지 않습니다. clone 후 빌드 전에 아래로 재생성합니다.

```bash
bun -F app ios:init       # iOS — xcodegen, cocoapods 필요
bun -F app android:init   # Android — Android SDK/NDK 필요
```

`ios:init`/`android:init`은 각각 `tauri ios/android init` 실행 직후
`bun run icons`(= `tauri icon app-icon.json`)를 자동으로 이어서 실행합니다.

⚠️ **왜 필요한가**: `tauri ios/android init`은 `gen/`을 만들 때 항상
**기본 Tauri 아이콘**을 심습니다. 이후 `tauri icon`을 실행하지 않으면 그
기본 아이콘이 그대로 남습니다. 특히 iOS 아이콘은 `icons/` 밑이 아니라
`gen/apple/Assets.xcassets/AppIcon.appiconset/`에 **직접** 쓰여지므로
(gen이 이미 존재할 때만 그렇게 동작합니다), init 이후 반드시 `tauri icon`을
다시 실행해야 합니다. `ios:init`/`android:init` 스크립트가 이 순서를 강제합니다.

소스오브트루스는 [`app-icon.svg`](./app-icon.svg)(아이콘 원본)와
[`app-icon.json`](./app-icon.json)(배경색 `#EFEEEB` 지정 manifest,
`tauri icon`의 입력)입니다. Android 어댑티브 아이콘 배경색은 `--ios-color`
CLI 옵션(기본값 `#fff`)이 그대로 적용되므로, 색을 코드 여러 곳에 흩어두지
않도록 `app-icon.json`의 `bg_color`로 고정했습니다. 아이콘을 다시 만들
때는 항상 `bun run icons`를 사용하고 `tauri icon app-icon.svg`를 직접
호출하지 마세요(직접 호출하면 배경색 플래그가 빠져 Android 배경이 `#fff`로
어긋납니다).

## iOS 시뮬레이터 개발 (정적 빌드 모드)

```bash
bun -F app ios:dev   # next build(정적 export) 후 시뮬레이터 실행
```

⚠️ **iOS에서 Next dev 서버(HMR) 모드를 쓰지 않는 이유**: Tauri CLI는 iOS
dev에서 `devUrl`(dev 서버)을 항상 `tauri://localhost` 커스텀 스킴으로
**프록시**합니다(IP 리터럴 devUrl로 바꿔도 동일). 이 환경에서 Next 16
(Turbopack) dev 런타임은 HTML·flight 데이터가 전부 도착해도 **hydration을
끝내지 못해** 화면은 보이지만 모든 버튼이 반응하지 않습니다. 반면 정적
export(`out/`)를 임베드해 서빙하면 동일한 `tauri://` 스킴에서도 hydration·
버튼·WASM 점역이 모두 정상 동작함을 확인했습니다.

그래서 `ios:dev`는 `--config src-tauri/tauri.sim.conf.json` 오버레이로
`devUrl`을 제거하고 `beforeDevCommand`를 `bun run build`로 바꿔, 매 실행마다
정적 export를 새로 빌드해 임베드합니다. HMR은 없지만 동작이 보장됩니다.

- **UI 빠른 반복**: 웹 브라우저에서 `bun -F app dev` (HMR 정상 동작)
- **시뮬레이터 확인**: `bun -F app ios:dev` (프론트 변경 시 재실행 필요)

## iOS 코드 서명 (개발자 팀)

iOS **실기기 빌드/배포**(`tauri ios build`, `tauri ios run`)는 Apple 코드 서명이
필요하며, Apple 개발자 **팀 ID**(10자리, 조직 단위 공유 값)를 지정해야 합니다.
아래 둘 중 하나로 제공합니다:

- 환경변수: `APPLE_DEVELOPMENT_TEAM=XXXXXXXXXX bun -F app tauri ios build`
- 또는 [`src-tauri/tauri.conf.json`](./src-tauri/tauri.conf.json)의
  `bundle.iOS.developmentTeam`에 실제 팀 ID 기입

> ⚠️ 이 값을 **빈 문자열 `""`로 두면 tauri 가 거부**합니다
> (`apple.development-team is empty`). 팀 ID가 정해지기 전에는 필드를 **아예 생략**하세요.

- **시뮬레이터**(`tauri ios dev`)는 ad-hoc 서명이라 팀 없이도 실행됩니다
  (`developmentTeam` 필드를 생략한 상태면 그대로 동작).
- 팀 ID 확인: Xcode → Settings → Accounts → 계정 선택 → 팀 목록의 10자리 ID,
  또는 <https://developer.apple.com> → Membership → Team ID.
