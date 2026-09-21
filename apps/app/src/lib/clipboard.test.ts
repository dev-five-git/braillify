import { afterEach, describe, expect, mock, test } from 'bun:test'

import {
  CLIPBOARD_ERROR_MESSAGE,
  copyText,
  EMPTY_CLIPBOARD_MESSAGE,
  writeClipboardText,
} from './clipboard'

const originalNavigator = globalThis.navigator

function setNavigator(value: unknown) {
  Object.defineProperty(globalThis, 'navigator', {
    configurable: true,
    value,
    writable: true,
  })
}

afterEach(() => {
  setNavigator(originalNavigator)
})

describe('copyText', () => {
  test('점역 결과를 클립보드 어댑터에 전달한다', async () => {
    const writeClipboardText = mock(async () => undefined)

    await expect(copyText('⠣⠒⠉⠻', writeClipboardText)).resolves.toBeUndefined()
    expect(writeClipboardText).toHaveBeenCalledWith('⠣⠒⠉⠻')
  })

  test('빈 결과는 클립보드에 쓰지 않는다', async () => {
    const writeClipboardText = mock(async () => undefined)

    await expect(copyText('', writeClipboardText)).rejects.toThrow(
      EMPTY_CLIPBOARD_MESSAGE,
    )
    expect(writeClipboardText).not.toHaveBeenCalled()
  })

  test('클립보드 어댑터 실패를 사용자용 오류로 변환한다', async () => {
    const writeClipboardText = mock(async () => {
      throw new Error('clipboard unavailable')
    })

    await expect(copyText('⠣⠒⠉⠻', writeClipboardText)).rejects.toThrow(
      CLIPBOARD_ERROR_MESSAGE,
    )
  })
})

describe('writeClipboardText 기본 어댑터', () => {
  test('Tauri 환경에서는 플러그인 writeText로 복사한다', async () => {
    const writeText = mock(async () => undefined)
    mock.module('@tauri-apps/plugin-clipboard-manager', () => ({ writeText }))

    await expect(
      writeClipboardText('⠣⠒⠉⠻', () => true),
    ).resolves.toBeUndefined()
    expect(writeText).toHaveBeenCalledWith('⠣⠒⠉⠻')
  })

  test('브라우저 환경에서는 navigator.clipboard로 복사한다', async () => {
    const writeText = mock(async () => undefined)
    setNavigator({ clipboard: { writeText } })

    await expect(
      writeClipboardText('⠣⠒⠉⠻', () => false),
    ).resolves.toBeUndefined()
    expect(writeText).toHaveBeenCalledWith('⠣⠒⠉⠻')
  })

  test('브라우저 클립보드를 쓸 수 없으면 오류를 던진다', async () => {
    setNavigator({})

    await expect(writeClipboardText('⠣⠒⠉⠻', () => false)).rejects.toThrow(
      '브라우저에서 클립보드를 사용할 수 없습니다.',
    )
  })
})

describe('copyText 기본 어댑터 통합', () => {
  test('어댑터를 주입하지 않으면 실제 클립보드 어댑터로 복사한다', async () => {
    const writeText = mock(async () => undefined)
    setNavigator({ clipboard: { writeText } })

    await expect(copyText('⠣⠒⠉⠻')).resolves.toBeUndefined()
    expect(writeText).toHaveBeenCalledWith('⠣⠒⠉⠻')
  })
})
