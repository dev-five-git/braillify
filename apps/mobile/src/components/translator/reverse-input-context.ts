'use client'

import { createContext, useContext } from 'react'

// 역점역 6점 입력기(client island)가 점역기 입력값을 읽고 갱신하기 위한 컨텍스트.
// 이 컨텍스트 덕분에 BrailleComposer의 정적 골격은 함수 prop 없이 서버에서 렌더될 수 있다.
export type ReverseInputValue = {
  onChange: (value: string) => void
  value: string
}

export const ReverseInputContext = createContext<ReverseInputValue | null>(null)

export function useReverseInput(): ReverseInputValue {
  const value = useContext(ReverseInputContext)
  if (!value) {
    throw new Error(
      'ReverseInputContext 바깥에서는 역점역 입력을 사용할 수 없습니다.',
    )
  }
  return value
}
