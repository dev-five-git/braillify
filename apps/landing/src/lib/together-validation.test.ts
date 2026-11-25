import { expect, test } from 'bun:test'

import { validateApplication } from './together-validation'

test('빈 필수 항목마다 안내를 반환한다', () => {
  expect(
    validateApplication({
      name: ' ',
      product: '',
      purpose: '',
      methods: [],
      consent: false,
      urlInvalid: false,
    }),
  ).toEqual({
    name: '이름을 입력해 주세요',
    product: '사용처 이름을 입력해 주세요',
    purpose: '사용 용도를 입력해 주세요',
    methods: '하나 이상 골라 주세요',
    consent: '동의해야 신청할 수 있어요',
  })
})

test('유효한 입력은 오류가 없고 잘못된 선택 주소는 거부한다', () => {
  const valid = {
    name: '학교',
    product: '앱',
    purpose: '교육',
    methods: ['Rust'],
    consent: true,
    urlInvalid: false,
  }
  expect(validateApplication(valid)).toEqual({})
  expect(validateApplication({ ...valid, urlInvalid: true })).toEqual({
    url: '올바른 주소를 입력해 주세요',
  })
})
