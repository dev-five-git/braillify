// Tauri 런타임 감지.
// @tauri-apps/api/core 의 isTauri() 와 동일한 판별식(`!!globalThis.isTauri`)을 사용하되,
// 브라우저 번들에 Tauri 패키지를 정적으로 포함하지 않도록 전역 플래그만 확인한다.
export function isTauriRuntime(): boolean {
  return Boolean((globalThis as { isTauri?: unknown }).isTauri)
}
