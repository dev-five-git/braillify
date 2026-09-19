/** Reports 점자세상 exact-match accuracy against the NIKL corpus reference. */

import { mkdir, readFile, readdir, writeFile } from 'node:fs/promises'
import { join } from 'node:path'

const ROOT = join(import.meta.dirname, '..')
const TEST_CASES_DIRECTORY = join(ROOT, 'test_cases')
const REPORT_PATH = join(ROOT, 'bench', 'JEOMJASESANG_CORPUS_BENCH.md')
const BRAILLE_BLANK = '\u2800'

interface CorpusCase {
  input: string
  unicode: string
  world?: string
}

function normalizeBraille(value: string): string {
  return value.replaceAll(' ', BRAILLE_BLANK)
}

/**
 * 점자세상은 한글 모드 API만 제공하므로, 영문으로 시작/끝나는 입력의
 * 외곽 영문 시작/끝 표지는 규칙 점역 결과와 분리해 비교한다.
 * 끝 표지(⠲)는 문장부호와 같으므로 실제 입력이 영문자로 끝날 때만 제거한다.
 */
function stripOuterEnglishMarkers(value: string, input: string): string {
  let normalized = value
  if (/^[A-Za-z]/.test(input)) normalized = normalized.replace(/^⠴/, '')
  if (/[A-Za-z]$/.test(input)) normalized = normalized.replace(/⠲$/, '')
  return normalized
}

interface YearStats {
  year: string
  total: number
  match: number
  mismatch: number
  missing: number
}

async function measureYear(directory: string, year: string): Promise<YearStats> {
  const corpusFiles = (await readdir(directory))
    .filter((file) => /^sentence_\d+\.json$/.test(file))
    .sort()
  const stats: YearStats = { year, total: 0, match: 0, mismatch: 0, missing: 0 }
  for (const file of corpusFiles) {
    const corpus = JSON.parse(
      await readFile(join(directory, file), 'utf8'),
    ) as CorpusCase[]
    for (const { input, unicode, world } of corpus) {
      stats.total++
      if (world == null || world === '') {
        stats.missing++
      } else if (
        stripOuterEnglishMarkers(normalizeBraille(world), input) ===
        stripOuterEnglishMarkers(unicode, input)
      ) {
        stats.match++
      } else {
        stats.mismatch++
      }
    }
  }
  return stats
}

function accuracyOf({ match, mismatch }: YearStats): number {
  const measured = match + mismatch
  return measured === 0 ? 0 : (match / measured) * 100
}

async function main(): Promise<void> {
  const corpusDirectories = (await readdir(TEST_CASES_DIRECTORY))
    .filter((entry) => /^\d{4}_corpus$/.test(entry))
    .sort()
  const perYear: YearStats[] = []
  for (const directory of corpusDirectories) {
    perYear.push(
      await measureYear(
        join(TEST_CASES_DIRECTORY, directory),
        directory.slice(0, 4),
      ),
    )
  }
  const grand = perYear.reduce<YearStats>(
    (acc, year) => ({
      year: '합계',
      total: acc.total + year.total,
      match: acc.match + year.match,
      mismatch: acc.mismatch + year.mismatch,
      missing: acc.missing + year.missing,
    }),
    { year: '합계', total: 0, match: 0, mismatch: 0, missing: 0 },
  )

  const row = (stats: YearStats): string =>
    `| ${stats.year} | ${stats.total} | ${stats.match + stats.mismatch} | ${stats.match} | ${stats.mismatch} | ${stats.missing} | ${accuracyOf(stats).toFixed(2)}% |`
  const report = [
    '# 점자세상 NIKL 병렬 말뭉치 정확도',
    '',
    '- 기준: NIKL Korean–Korean Braille Parallel Corpus (연도별 v1.0)',
    '- 방식: 점자 공백을 정규화한 뒤 문장 단위 완전 일치 비교',
    '- 미수집 문장은 측정 대상에서 제외한다.',
    '',
    '| 연도 | 전체 문장 | 측정 대상 | 일치 | 불일치 | 미수집 | 완전 일치율 |',
    '|---|---:|---:|---:|---:|---:|---:|',
    ...perYear.map(row),
    row(grand),
    '',
  ].join('\n')

  await mkdir(join(ROOT, 'bench'), { recursive: true })
  await writeFile(REPORT_PATH, report, 'utf8')
  for (const stats of perYear) {
    console.log(
      `점자세상 ${stats.year}: ${stats.match}/${stats.match + stats.mismatch} (${accuracyOf(stats).toFixed(2)}%), ${stats.missing} missing`,
    )
  }
  console.log(
    `점자세상 합계: ${grand.match}/${grand.match + grand.mismatch} (${accuracyOf(grand).toFixed(2)}%), ${grand.missing} missing`,
  )
}

main().catch((error: unknown) => {
  console.error(error)
  process.exit(1)
})
