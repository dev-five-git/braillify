import { expect, test } from 'bun:test'

import { previewDescription } from './together-preview'

test('스크린리더 미리 보기에 이름, 구분과 점역을 담는다', () => {
  expect(previewDescription('점자 학교', '학교', '⠨⠎')).toBe(
    '미리 보기: 점자 학교, 학교, 점자 ⠨⠎',
  )
})

test('이름이 비어 있으면 안내를 읽는다', () => {
  expect(previewDescription('', '기업', '')).toContain(
    '이름을 입력하면 여기에 표시돼요',
  )
})
