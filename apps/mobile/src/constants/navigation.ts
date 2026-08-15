export type AppView = 'translator' | 'editor' | 'history'

export const SIDEBAR_ITEMS = [
  {
    description: '점역 · 역점역',
    id: 'translator',
    label: '점역기',
    symbol: '⠿',
  },
  {
    description: '직접 점자 편집',
    id: 'editor',
    label: '점자 편집기',
    symbol: '⠶',
  },
  {
    description: '최근 작업 · 즐겨찾기',
    id: 'history',
    label: '히스토리',
    symbol: '⠷',
  },
] as const satisfies ReadonlyArray<{
  description: string
  id: AppView
  label: string
  symbol: string
}>
