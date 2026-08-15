import { useCallback, useEffect, useState } from 'react'

import {
  addHistoryEntry,
  clearHistory,
  deleteHistoryEntry,
  type HistoryEntry,
  type HistoryEntryDraft,
  loadHistory,
  toggleFavoriteHistoryEntry,
} from '@/lib/history'

export function useTranslationHistory() {
  const [entries, setEntries] = useState<HistoryEntry[]>([])

  useEffect(() => {
    setEntries(loadHistory())
  }, [])

  const addEntry = useCallback((draft: HistoryEntryDraft) => {
    setEntries(addHistoryEntry(draft))
  }, [])

  const deleteEntry = useCallback((id: string) => {
    setEntries(deleteHistoryEntry(id))
  }, [])

  const deleteAll = useCallback(() => {
    setEntries(clearHistory())
  }, [])

  const toggleFavorite = useCallback((id: string) => {
    setEntries(toggleFavoriteHistoryEntry(id))
  }, [])

  return {
    addEntry,
    deleteAll,
    deleteEntry,
    entries,
    toggleFavorite,
  }
}
