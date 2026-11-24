import { expect, test } from 'bun:test'
import { existsSync } from 'node:fs'
import { mkdir, mkdtemp, readFile, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import { brailleSvg, escapeHtml, pruneTogetherImages, translateName } from './together-images'

const cli = join(import.meta.dir, '..', 'target', 'release', process.platform === 'win32' ? 'braillify.exe' : 'braillify')

test.skipIf(!existsSync(cli))('하이픈으로 시작하는 이름도 점역한다', () => {
  expect(translateName(cli, '-학교')).toMatch(/^[\u2800-\u28ff]+$/u)
})

test('점역 셀의 볼록한 점만 그린다', () => {
  const svg = brailleSvg('⠁⠀⠿')
  expect((svg.match(/<circle /g) || []).length).toBe(7)
  expect(() => brailleSvg('A')).toThrow()
})

test('긴 점자는 960px 안에 맞추고 점 지름을 4px 이상 유지한다', () => {
  const svg = brailleSvg('⠿'.repeat(100))
  const width = Number(/width="([\d.]+)"/.exec(svg)?.[1])
  const radius = Number(/ r="([\d.]+)"/.exec(svg)?.[1])
  expect(width).toBeLessThanOrEqual(960)
  expect(radius).toBeGreaterThanOrEqual(2)
})

test('공개 이름이 HTML로 실행되지 않는다', () => {
  expect(escapeHtml('<script>"&')).toBe('&lt;script&gt;&quot;&amp;')
})

test('게시 중단된 곳의 생성 이미지만 정리한다', async () => {
  const root = await mkdtemp(join(tmpdir(), 'braillify-prune-'))
  const cert = join(root, 'together', 'cert')
  await mkdir(cert, { recursive: true })
  await writeFile(join(cert, 'org-123.png'), 'old')
  await writeFile(join(cert, 'unrelated.png'), 'keep')
  await pruneTogetherImages([], root)
  await expect(readFile(join(cert, 'org-123.png'))).rejects.toThrow()
  expect(await readFile(join(cert, 'unrelated.png'), 'utf8')).toBe('keep')
})
