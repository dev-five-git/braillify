import { expect, test } from 'bun:test'

import { normalizeFracBraces } from './normalizeFracBraces'

test('한 글자 인자를 중괄호로 감싼다', () => {
  expect(normalizeFracBraces('\\frac34')).toBe('\\frac{3}{4}')
  expect(normalizeFracBraces('\\frac{3}4')).toBe('\\frac{3}{4}')
})

test('중첩 분수도 재귀적으로 정규화한다', () => {
  expect(normalizeFracBraces('\\frac{\\frac12}3')).toBe(
    '\\frac{\\frac{1}{2}}{3}',
  )
})

test('제어 시퀀스 인자를 통째로 감싼다', () => {
  expect(normalizeFracBraces('\\frac\\pi2')).toBe('\\frac{\\pi}{2}')
})

test('\\fracture 처럼 접두어가 겹치는 명령은 건드리지 않는다', () => {
  expect(normalizeFracBraces('\\fracture')).toBe('\\fracture')
})

test('닫는 중괄호가 없어도 입력을 잃지 않는다', () => {
  expect(normalizeFracBraces('\\frac{12')).toBe('\\frac{12}{}')
})

test('분수가 없는 수식은 그대로 둔다', () => {
  expect(normalizeFracBraces('x^{2}+1')).toBe('x^{2}+1')
})
