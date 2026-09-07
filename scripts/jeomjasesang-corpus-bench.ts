/** Reports 점자세상 exact-match accuracy against the NIKL corpus reference. */

import { mkdir, readFile, readdir, writeFile } from 'node:fs/promises'
import { join } from 'node:path'

const ROOT = join(import.meta.dirname, '..')
const CORPUS_DIRECTORY = join(ROOT, 'test_cases', 'corpus')
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

async function main(): Promise<void> {
  const corpusFiles = (await readdir(CORPUS_DIRECTORY))
    .filter((file) => /^sentence_\d+\.json$/.test(file))
    .sort()
  const corpus = (
    await Promise.all(
      corpusFiles.map(
        async (file) =>
          JSON.parse(
            await readFile(join(CORPUS_DIRECTORY, file), 'utf8'),
          ) as CorpusCase[],
      ),
    )
  ).flat()
  let match = 0
  let mismatch = 0
  let missing = 0
  for (const { input, unicode, world } of corpus) {
    if (world == null) {
      missing++
    } else if (
      stripOuterEnglishMarkers(normalizeBraille(world), input) ===
      stripOuterEnglishMarkers(unicode, input)
    ) {
      match++
    } else {
      mismatch++
    }
  }

  const measured = match + mismatch
  const accuracy = measured === 0 ? 0 : (match / measured) * 100
  const report = [
    '# 점자세상 NIKL 병렬 말뭉치 정확도',
    '',
    '- 기준: NIKL Korean–Korean Braille Parallel Corpus 2025 v1.0',
    '- 방식: 점자 공백을 정규화한 뒤 문장 단위 완전 일치 비교',
    '',
    '| 항목 | 값 |',
    '|---|---:|',
    `| 전체 문장 | ${corpus.length} |`,
    `| API 응답 수 | ${measured} |`,
    `| 측정 대상 | ${measured} |`,
    `| 일치 | ${match} |`,
    `| 불일치 | ${mismatch} |`,
    `| 미수집 | ${missing} |`,
    `| **문장 단위 완전 일치율** | **${accuracy.toFixed(2)}%** |`,
    '',
  ].join('\n')

  await mkdir(join(ROOT, 'bench'), { recursive: true })
  await writeFile(REPORT_PATH, report, 'utf8')
  console.log(`점자세상: ${match}/${measured} (${accuracy.toFixed(2)}%), ${missing} missing`)
}

main().catch((error: unknown) => {
  console.error(error)
  process.exit(1)
})
