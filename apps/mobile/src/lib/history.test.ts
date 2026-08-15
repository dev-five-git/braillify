import { describe, expect, test } from 'bun:test'

import {
  addHistoryEntry,
  clearHistory,
  deleteHistoryEntry,
  HISTORY_STORAGE_KEY,
  loadHistory,
  MAX_HISTORY_ENTRIES,
  toggleFavoriteHistoryEntry,
} from './history'

class MemoryStorage {
  private values = new Map<string, string>()

  getItem(key: string) {
    return this.values.get(key) ?? null
  }

  removeItem(key: string) {
    this.values.delete(key)
  }

  setItem(key: string, value: string) {
    this.values.set(key, value)
  }
}

describe('translation history', () => {
  test('최근 변환을 모드와 함께 저장하고 다시 읽는다', () => {
    const storage = new MemoryStorage()

    addHistoryEntry(
      { input: '안녕', mode: 'general', result: '⠣⠒⠉⠻' },
      storage,
      () => 'entry-1',
      () => new Date('2026-07-27T12:00:00.000Z'),
    )

    expect(loadHistory(storage)).toEqual([
      {
        createdAt: '2026-07-27T12:00:00.000Z',
        favorite: false,
        id: 'entry-1',
        input: '안녕',
        mode: 'general',
        result: '⠣⠒⠉⠻',
      },
    ])
  })

  test('상한을 넘는 일반 기록은 오래된 것부터 제거한다', () => {
    const storage = new MemoryStorage()
    const total = MAX_HISTORY_ENTRIES + 5

    for (let index = 0; index < total; index += 1) {
      addHistoryEntry(
        {
          input: `입력 ${index}`,
          mode: 'general',
          result: `결과 ${index}`,
        },
        storage,
        () => `entry-${index}`,
      )
    }

    const entries = loadHistory(storage)
    expect(entries).toHaveLength(MAX_HISTORY_ENTRIES)
    expect(entries[0]?.id).toBe(`entry-${total - 1}`)
    expect(entries.at(-1)?.id).toBe(`entry-${total - MAX_HISTORY_ENTRIES}`)
  })

  test('상한을 넘어도 즐겨찾기 기록은 유지된다', () => {
    const storage = new MemoryStorage()

    addHistoryEntry(
      { input: '오래된 즐겨찾기', mode: 'general', result: '⠐' },
      storage,
      () => 'favorite-entry',
    )
    toggleFavoriteHistoryEntry('favorite-entry', storage)

    for (let index = 0; index < MAX_HISTORY_ENTRIES + 10; index += 1) {
      addHistoryEntry(
        { input: `입력 ${index}`, mode: 'general', result: `결과 ${index}` },
        storage,
        () => `entry-${index}`,
      )
    }

    const favorite = loadHistory(storage).find(
      (entry) => entry.id === 'favorite-entry',
    )
    expect(favorite?.favorite).toBe(true)
  })

  test('손상된 저장 데이터는 삭제하고 빈 상태로 복구한다', () => {
    const storage = new MemoryStorage()
    storage.setItem(HISTORY_STORAGE_KEY, '{invalid json')

    expect(loadHistory(storage)).toEqual([])
    expect(storage.getItem(HISTORY_STORAGE_KEY)).toBeNull()
  })

  test('저장소가 없는 서버 환경에서는 빈 상태를 반환한다', () => {
    expect(loadHistory()).toEqual([])
  })

  test('배열이 아니거나 유효하지 않은 항목은 기록으로 불러오지 않는다', () => {
    const storage = new MemoryStorage()
    storage.setItem(HISTORY_STORAGE_KEY, '{}')

    expect(loadHistory(storage)).toEqual([])
    expect(storage.getItem(HISTORY_STORAGE_KEY)).toBeNull()

    storage.setItem(HISTORY_STORAGE_KEY, '[null]')
    expect(loadHistory(storage)).toEqual([])
  })

  test('기본 생성기로 식별자와 시각을 기록한다', () => {
    const storage = new MemoryStorage()
    const [entry] = addHistoryEntry(
      { input: '안녕', mode: 'general', result: '⠣⠒⠉⠻' },
      storage,
    )

    expect(entry?.id).toBeString()
    expect(Number.isNaN(Date.parse(entry?.createdAt ?? ''))).toBe(false)
  })

  test('저장소 접근이 모두 실패해도 점역 기록 흐름을 중단하지 않는다', () => {
    const storage = {
      getItem() {
        throw new Error('storage unavailable')
      },
      removeItem() {
        throw new Error('storage unavailable')
      },
      setItem() {
        throw new Error('storage unavailable')
      },
    }

    expect(loadHistory(storage)).toEqual([])
    expect(
      addHistoryEntry(
        { input: '안녕', mode: 'general', result: '⠣⠒⠉⠻' },
        storage,
        () => 'offline-entry',
        () => new Date('2026-07-27T12:00:00.000Z'),
      ),
    ).toEqual([
      {
        createdAt: '2026-07-27T12:00:00.000Z',
        favorite: false,
        id: 'offline-entry',
        input: '안녕',
        mode: 'general',
        result: '⠣⠒⠉⠻',
      },
    ])
    expect(clearHistory(storage)).toEqual([])
  })

  test('개별 삭제와 전체 삭제를 저장소에 반영한다', () => {
    const storage = new MemoryStorage()
    addHistoryEntry(
      { input: '안녕', mode: 'general', result: '⠣⠒⠉⠻' },
      storage,
      () => 'general-entry',
    )
    addHistoryEntry(
      { input: '$x$', mode: 'math', result: '⠭' },
      storage,
      () => 'math-entry',
    )

    expect(deleteHistoryEntry('general-entry', storage)).toHaveLength(1)
    expect(clearHistory(storage)).toEqual([])
    expect(loadHistory(storage)).toEqual([])
  })

  test('즐겨찾기 상태를 켜고 끄며 기존 기록도 호환한다', () => {
    const storage = new MemoryStorage()
    storage.setItem(
      HISTORY_STORAGE_KEY,
      JSON.stringify([
        {
          createdAt: '2026-07-27T12:00:00.000Z',
          id: 'legacy-entry',
          input: '안녕',
          mode: 'general',
          result: '⠣⠒⠉⠻',
        },
      ]),
    )

    expect(loadHistory(storage)[0]?.favorite).toBe(false)
    expect(
      toggleFavoriteHistoryEntry('legacy-entry', storage)[0]?.favorite,
    ).toBe(true)
    expect(
      toggleFavoriteHistoryEntry('legacy-entry', storage)[0]?.favorite,
    ).toBe(false)
  })

  test('역점역 기록을 유효한 기록으로 다시 읽는다', () => {
    const storage = new MemoryStorage()
    addHistoryEntry(
      { input: '⠣⠒⠉⠻', mode: 'reverse', result: '안녕' },
      storage,
      () => 'reverse-entry',
    )

    expect(loadHistory(storage)[0]).toMatchObject({
      id: 'reverse-entry',
      mode: 'reverse',
      result: '안녕',
    })
  })
})
