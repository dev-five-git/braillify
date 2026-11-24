import { expect, test } from 'bun:test'
import { existsSync } from 'node:fs'
import { mkdir, mkdtemp, readFile, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import { brailleSvg, escapeHtml, generateTogetherImages, pruneTogetherImages, translateName } from './together-images'

const cli = join(import.meta.dir, '..', 'target', 'release', process.platform === 'win32' ? 'braillify.exe' : 'braillify')

test('승인된 곳의 인증 이미지와 배지의 크기를 생성한다', async () => {
  const root = await mkdtemp(join(tmpdir(), 'braillify-render-'))
  const logos = join(root, 'images', 'together')
  await mkdir(logos, { recursive: true })
  await writeFile(join(logos, 'asset-123.png'), await readFile(join(import.meta.dir, '../apps/landing/public/og-image.png')))
  await generateTogetherImages([{ issue: 123, name: '점자 학교', kind: '학교', logo: '/images/together/asset-123.png', approvedAt: '2026-11-20' }], root)
  for (const [file, width, height] of [
    ['cert/org-123.png', 1200, 630], ['cert/org-123@2x.png', 2400, 1260],
    ['badge/org-123-light.png', 300, 80], ['badge/org-123-dark.png', 300, 80],
    ['badge/org-123-light@2x.png', 600, 160], ['badge/org-123-dark@2x.png', 600, 160],
  ] as const) {
    const png = await readFile(join(root, 'together', file))
    expect(png.subarray(0, 8)).toEqual(Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]))
    expect([png.readUInt32BE(16), png.readUInt32BE(20)]).toEqual([width, height])
  }
}, 120_000)

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
  await writeFile(join(cert, 'org-124.png'), 'approved')
  await writeFile(join(cert, 'unrelated.png'), 'keep')
  await pruneTogetherImages([{ issue: 124, name: '학교', kind: '학교', logo: '/images/together/asset-124.png', approvedAt: '2026-11-20' }], root)
  await expect(readFile(join(cert, 'org-123.png'))).rejects.toThrow()
  expect(await readFile(join(cert, 'org-124.png'), 'utf8')).toBe('approved')
  expect(await readFile(join(cert, 'unrelated.png'), 'utf8')).toBe('keep')
})
