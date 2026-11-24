# 함께하는 곳 목록

신청 이슈는 공개 데이터가 아니다. 실제 사용과 공개 동의, 로고를 확인한 뒤 관리자가 PR에서 `together.json`에 승인 순서대로 항목을 추가한다. 이슈 번호는 중복할 수 없다.

```json
{
  "issue": 123,
  "kind": "학교",
  "name": "이름",
  "logo": "/images/together/asset-123.png",
  "approvedAt": "2026-11-20"
}
```

로고는 `public/images/together/`의 밝은 바탕용 PNG 또는 JPG 한 개로, 2 MiB 이하여야 한다. 빌드는 입력의 공개 필드만 `public/together.json`으로 생성하며 잘못된 기록이나 로고는 거부한다. 게시 중단 요청 시 해당 항목을 제거한다.

빌드에는 `cargo build --release -p braillify`로 만든 점역 CLI와 Chrome이 필요하다. 인증 이미지와 밝은·어두운 배지는 빌드에서 생성한다. main 배포가 끝난 뒤 공개 목록과 여섯 PNG가 실제로 제공되는지 확인하고, 아직 답글이 없는 이슈에만 GitHub Actions가 답글을 단다.
