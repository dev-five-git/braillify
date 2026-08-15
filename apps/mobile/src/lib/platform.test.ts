import { afterEach, describe, expect, test } from 'bun:test'

import { isTauriRuntime } from './platform'

describe('isTauriRuntime', () => {
  afterEach(() => {
    delete (globalThis as { isTauri?: unknown }).isTauri
  })

  test('전역 isTauri 플래그가 있으면 Tauri 런타임으로 판별한다', () => {
    ;(globalThis as { isTauri?: unknown }).isTauri = true

    expect(isTauriRuntime()).toBe(true)
  })

  test('전역 isTauri 플래그가 없으면 브라우저 런타임으로 판별한다', () => {
    delete (globalThis as { isTauri?: unknown }).isTauri

    expect(isTauriRuntime()).toBe(false)
  })
})
