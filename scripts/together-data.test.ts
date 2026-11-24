import { expect, test } from 'bun:test'
import { mkdir, mkdtemp, readFile, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import { publishPartners } from './together-publish'
import { validatePartners } from './together-data'

const approved = {
  issue: 123,
  kind: '학교',
  name: '점자 학교',
  logo: '/images/together/asset-123.png',
  approvedAt: '2026-11-20',
}

test('승인 순서와 공개 항목을 보존한다', () => {
  expect(validatePartners([approved])).toEqual([approved])
  expect(validatePartners([{ ...approved, privateNotes: 'not public' }])).toEqual([approved])
})

test('중복 이슈와 외부 또는 경로 이탈 로고를 거부한다', () => {
  expect(() => validatePartners([approved, approved])).toThrow()
  expect(() => validatePartners([{ ...approved, logo: 'https://example.com/a.png' }])).toThrow()
  expect(() => validatePartners([{ ...approved, logo: '/images/together/../secret.png' }])).toThrow()
})

test('유효하지 않은 이름, 구분, 승인 일자와 이슈 번호를 거부한다', () => {
  expect(() => validatePartners([{ ...approved, name: '가'.repeat(21) }])).toThrow()
  expect(() => validatePartners([{ ...approved, kind: '기타' }])).toThrow()
  expect(() => validatePartners([{ ...approved, approvedAt: '2026-02-30' }])).toThrow()
  expect(() => validatePartners([{ ...approved, issue: -1 }])).toThrow()
})

test('잘못된 로고는 이전 공개 목록을 덮어쓰지 않는다', async () => {
  const publicDir = await mkdtemp(join(tmpdir(), 'braillify-together-'))
  const output = join(publicDir, 'together.json')
  const logos = join(publicDir, 'images', 'together')
  await mkdir(logos, { recursive: true })
  await writeFile(join(logos, 'asset-123.png'), 'not a PNG')
  await writeFile(output, 'previous')
  await expect(publishPartners([approved], publicDir)).rejects.toThrow()
  expect(await readFile(output, 'utf8')).toBe('previous')
})

test('확인된 로고와 공개 기록으로 목록을 만든다', async () => {
  const publicDir = await mkdtemp(join(tmpdir(), 'braillify-together-'))
  const logos = join(publicDir, 'images', 'together')
  await mkdir(logos, { recursive: true })
  await writeFile(join(logos, 'asset-123.png'), Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]))
  await publishPartners([approved], publicDir)
  expect(JSON.parse(await readFile(join(publicDir, 'together.json'), 'utf8'))).toEqual([approved])
})

test('게시 중단된 로고는 공유하지 않을 때 제거해야 한다', async () => {
  const publicDir = await mkdtemp(join(tmpdir(), 'braillify-together-'))
  const logos = join(publicDir, 'images', 'together')
  await mkdir(logos, { recursive: true })
  await writeFile(join(logos, 'asset-123.png'), Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]))
  await expect(publishPartners([], publicDir)).rejects.toThrow('Unreferenced')
  await publishPartners([approved, { ...approved, issue: 124 }], publicDir)
  expect(JSON.parse(await readFile(join(publicDir, 'together.json'), 'utf8'))).toHaveLength(2)
})
