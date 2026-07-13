import { TranslateMode } from './translate.js'

export type HistoryItem = {
  id: string
  source: string
  braille: string
  mode: TranslateMode
  favorite: boolean
  createdAt: number
}

export interface HistoryAdapter {
  getItem(key: string): string | null
  setItem(key: string, value: string): void
}

class LocalStorageAdapter implements HistoryAdapter {
  getItem(key: string): string | null {
    if (typeof window !== 'undefined' && window.localStorage) {
      return window.localStorage.getItem(key)
    }
    return null
  }
  setItem(key: string, value: string): void {
    if (typeof window !== 'undefined' && window.localStorage) {
      window.localStorage.setItem(key, value)
    }
  }
}

const KEY = 'braillify.history.v1'
let currentAdapter: HistoryAdapter = new LocalStorageAdapter()

export function setHistoryAdapter(adapter: HistoryAdapter) {
  currentAdapter = adapter
}

type Listener = () => void
const listeners = new Set<Listener>()

function emit() {
  for (const fn of listeners) fn()
}

function read(): HistoryItem[] {
  try {
    const raw = currentAdapter.getItem(KEY)
    if (!raw) return []
    const parsed = JSON.parse(raw)
    return Array.isArray(parsed) ? parsed : []
  } catch {
    return []
  }
}

function write(items: HistoryItem[]): void {
  currentAdapter.setItem(KEY, JSON.stringify(items))
  emit()
}

export function listHistory(): HistoryItem[] {
  return read()
}

export function subscribeHistory(fn: Listener): () => void {
  listeners.add(fn)
  return () => listeners.delete(fn)
}

export function pushHistory(input: {
  source: string
  braille: string
  mode: TranslateMode
}): HistoryItem {
  const item: HistoryItem = {
    id:
      typeof crypto !== 'undefined' && crypto.randomUUID
        ? crypto.randomUUID()
        : Math.random().toString(36).substring(2, 15),
    source: input.source,
    braille: input.braille,
    mode: input.mode,
    favorite: false,
    createdAt: Date.now(),
  }

  const next = [item, ...read()]
  write(next)

  return item
}

export function toggleFavorite(id: string): void {
  const next = read().map((it) =>
    it.id === id ? { ...it, favorite: !it.favorite } : it,
  )
  write(next)
}

export function removeHistory(id: string): void {
  const next = read().filter((it) => it.id !== id)
  write(next)
}
