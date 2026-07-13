function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

export async function copyText(text: string): Promise<void> {
  if (isTauri()) {
    try {
      // tauri의 clipboard-manager를 동적으로 로드하여 비Tauri 환경 빌드 에러 방지
      const { writeText: tauriWriteText } =
        await import('@tauri-apps/plugin-clipboard-manager')
      await tauriWriteText(text)
      return
    } catch {
      // ignore, fallback to navigator
    }
  }
  if (typeof navigator !== 'undefined' && navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text)
    return
  }
  throw new Error('Clipboard API unavailable')
}
