import { expect, test } from 'bun:test'
import { existsSync } from 'node:fs'
import { mkdir, mkdtemp, readFile, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { spawnSync } from 'node:child_process'

import { brailleSvg, chromeFailure, escapeHtml, pruneTogetherImages, translateName, withChromeProfile } from './together-image-utils'

const cli = join(import.meta.dir, '..', 'target', 'release', process.platform === 'win32' ? 'braillify.exe' : 'braillify')
const chrome = process.env.CHROME_PATH || Bun.which('google-chrome') || Bun.which('chromium') || (process.platform === 'win32' ? 'C:/Program Files/Google/Chrome/Application/chrome.exe' : process.platform === 'darwin' ? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' : '')
const canRender = existsSync(cli) && !!chrome && (!!Bun.which(chrome) || existsSync(chrome))

test('Chrome 실패 진단은 프로세스 상태와 비공개 정보가 제거된 stderr를 포함한다', () => {
  const result = spawnSync(process.execPath, ['-e', 'process.stderr.write("[ERROR:renderer.cc(42)] secret /private/path\\n"); process.exit(7)'], { encoding: 'utf8' })
  const message = chromeFailure(result, 'badge.png', 123, true)
  expect(message).toContain('badge.png')
  expect(message).toContain('123ms')
  expect(message).toContain('status=7')
  expect(message).toContain('outputExists=true')
  expect(message).toContain('ERROR:renderer.cc(42)')
  expect(message).not.toContain('secret')
  expect(message).not.toContain('/private/path')
  const timedOut = spawnSync(process.execPath, ['-e', 'setTimeout(() => {}, 2000)'], { encoding: 'utf8', timeout: 100 })
  const timeoutMessage = chromeFailure(timedOut, 'cert.png', 100, false)
  expect(timeoutMessage).toContain('status=null')
  expect(timeoutMessage).toContain('error=ETIMEDOUT')
  expect(timeoutMessage).toContain('outputExists=false')
  expect(timeoutMessage).toContain('stderr=(no recognized diagnostics)')
})

test('Chrome 프로필은 성공과 실패 직후 정리한다', async () => {
  const work = await mkdtemp(join(tmpdir(), 'braillify-profiles-'))
  let first = ''
  expect(await withChromeProfile(work, async (profile) => {
    first = profile
    await writeFile(join(profile, 'data'), 'temporary')
    return 'done'
  })).toBe('done')
  expect(existsSync(first)).toBe(false)
  let second = ''
  await expect(withChromeProfile(work, async (profile) => {
    second = profile
    await writeFile(join(profile, 'data'), 'temporary')
    throw new Error('render failed')
  })).rejects.toThrow('render failed')
  expect(second).not.toBe(first)
  expect(existsSync(second)).toBe(false)
  expect(existsSync(work)).toBe(true)
})

test.skipIf(!canRender && !process.env.CI)('승인된 곳의 인증 이미지와 배지의 크기를 생성한다', async () => {
  const { generateTogetherImages } = await import('./together-images')
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
}, 300_000)

test.skipIf(!existsSync(cli))('하이픈으로 시작하는 이름도 점역한다', () => {
  expect(translateName(cli, '-학교')).toMatch(/^[\u2800-\u28ff]+$/u)
})

test.skipIf(!existsSync(cli))('Chrome 실행 오류의 원인을 표시한다', async () => {
  const { generateTogetherImages } = await import('./together-images')
  const root = await mkdtemp(join(tmpdir(), 'braillify-render-error-'))
  const logos = join(root, 'images', 'together')
  await mkdir(logos, { recursive: true })
  await writeFile(join(logos, 'asset-123.png'), await readFile(join(import.meta.dir, '../apps/landing/public/og-image.png')))
  const previous = process.env.CHROME_PATH
  process.env.CHROME_PATH = join(root, 'missing-chrome')
  try {
    await expect(generateTogetherImages([{ issue: 123, name: '학교', kind: '학교', logo: '/images/together/asset-123.png', approvedAt: '2026-11-20' }], root)).rejects.toThrow(/Chrome logo check failed:.*missing-chrome/)
  } finally {
    if (previous === undefined) delete process.env.CHROME_PATH
    else process.env.CHROME_PATH = previous
  }
})

test('점역 실행 파일이 없으면 오류를 반환한다', () => {
  expect(() => translateName(join(tmpdir(), 'missing-braillify-executable'), '학교')).toThrow('Braille generation failed')
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
