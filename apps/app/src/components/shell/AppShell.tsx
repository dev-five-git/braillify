'use client'

import { Grid, VStack } from '@devup-ui/react'
import { createContext, type ReactNode, useContext, useState } from 'react'

import { AppSidebar } from '@/components/navigation/AppSidebar'
import { BottomTabBar } from '@/components/navigation/BottomTabBar'
import type { AppView } from '@/constants/navigation'
import type { TranslateMode } from '@/constants/translation'
import { useTranslationHistory } from '@/hooks/useTranslationHistory'
import type { HistoryEntry, HistoryEntryDraft } from '@/lib/history'

// 히스토리 항목을 점역기로 되돌릴 때 넘기는 초안. requestId 로 같은 항목을
// 다시 불러올 때도 effect 가 재실행된다.
export type TranslationDraft = {
  input: string
  mode: TranslateMode
  requestId: number
  result: string
}

type AppState = {
  activeView: AppView
  addEntry: (draft: HistoryEntryDraft) => void
  deleteAll: () => void
  deleteEntry: (id: string) => void
  entries: HistoryEntry[]
  navigate: (view: AppView) => void
  requestRestore: (entry: HistoryEntry) => void
  restoreDraft: TranslationDraft | null
  toggleFavorite: (id: string) => void
}

const AppStateContext = createContext<AppState | null>(null)

export function useAppState(): AppState {
  const state = useContext(AppStateContext)
  if (!state) {
    throw new Error('AppShell 바깥에서는 앱 상태를 사용할 수 없습니다.')
  }
  return state
}

// 앱 전역의 activeView·복원·히스토리 상태를 소유하는 client boundary.
// 각 화면(children)의 정적 골격은 서버에서 조합되어 내려온다.
export function AppShell({ children }: { children: ReactNode }) {
  const [activeView, setActiveView] = useState<AppView>('translator')
  const [restoreDraft, setRestoreDraft] = useState<TranslationDraft | null>(
    null,
  )
  const [restoreRequestId, setRestoreRequestId] = useState(0)
  const { addEntry, deleteAll, deleteEntry, entries, toggleFavorite } =
    useTranslationHistory()

  const requestRestore = (entry: HistoryEntry) => {
    const nextRequestId = restoreRequestId + 1
    setRestoreRequestId(nextRequestId)
    setRestoreDraft({
      input: entry.input,
      mode: entry.mode,
      requestId: nextRequestId,
      result: entry.result,
    })
    setActiveView('translator')
  }

  return (
    <AppStateContext
      value={{
        activeView,
        addEntry,
        deleteAll,
        deleteEntry,
        entries,
        navigate: setActiveView,
        requestRestore,
        restoreDraft,
        toggleFavorite,
      }}
    >
      <Grid
        bg="$background"
        gridTemplateColumns={[
          'minmax(0, 1fr)',
          null,
          '280px minmax(0, 1fr)',
          null,
          '320px minmax(0, 1fr)',
        ]}
        minH="100dvh"
      >
        <AppSidebar activeView={activeView} onNavigate={setActiveView} />
        <VStack
          as="main"
          gap="28px"
          minW="0"
          pb={['96px', null, '48px']}
          pt={['max(env(safe-area-inset-top, 0px), 59px)', null, '48px']}
          px={['20px', null, '48px', null, '60px']}
        >
          {children}
        </VStack>
        <BottomTabBar activeView={activeView} onNavigate={setActiveView} />
      </Grid>
    </AppStateContext>
  )
}
