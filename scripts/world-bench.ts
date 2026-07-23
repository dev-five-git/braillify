/**
 * Benchmark: 점자세상 (braillekorea.org) 정답률 측정.
 *
 * test_cases/**.json 의 모든 entry 를 순회하며, `world` 필드 (점자세상 API 응답)
 * 가 PDF 정답 (`unicode` 필드) 과 얼마나 일치하는지 통계를 낸다.
 *
 * 비교는 단순 문자열 동치. PDF 정답이 braillify 의 ground truth 이므로
 * 두 외부 점역기가 같은 PDF 정답을 얼마나 따르는지 객관적으로 비교 가능하다.
 *
 * Skip 정책 (fetch-world.ts 와 동일):
 *  - note === "LaTeX" : 동일 input 의 LaTeX 변형 → 의미적 중복, 제외
 *  - input 이 비어있음 : 제외
 *  - world 가 비어있음 : (fetch 에러 또는 skip 표식) 제외
 *  - unicode 가 비어있음 : (대문자 수학 변수 등 base64 패턴 외) → 제외
 *
 * Usage:
 *   bun run scripts/world-bench.ts
 *
 * Output:
 *   - bench/WORLD_BENCH.md (사람용 보고서)
 *   - 표준 출력 요약
 */

import { mkdir, readdir, readFile, writeFile } from 'node:fs/promises'
import { dirname, join } from 'node:path'

interface TestCaseEntry {
  input: string
  internal?: string
  expected?: string
  unicode?: string
  alternatives?: TestAlternative[]
  world?: string
  jeomsarang?: string
  note?: string
  context?: string
}

interface TestAlternative {
  internal: string
  expected: string
  unicode: string
}

interface CategoryStats {
  total: number
  measured: number
  skipped_latex: number
  skipped_empty_input: number
  skipped_no_world: number
  skipped_no_unicode: number
  match: number
  mismatch: number
  mismatches: Array<{
    file: string
    line: number
    input: string
    pdf: string
    world: string
  }>
}

const SCRIPT_DIR = import.meta.dirname
const TEST_CASES_DIR = join(SCRIPT_DIR, '..', 'test_cases')
const REPORT_PATH = join(SCRIPT_DIR, '..', 'bench', 'WORLD_BENCH.md')
const MISMATCH_PATH = join(SCRIPT_DIR, '..', 'bench', 'WORLD_MISMATCHES.md')

function newStats(): CategoryStats {
  return {
    total: 0,
    measured: 0,
    skipped_latex: 0,
    skipped_empty_input: 0,
    skipped_no_world: 0,
    skipped_no_unicode: 0,
    match: 0,
    mismatch: 0,
    mismatches: [],
  }
}

function add(into: CategoryStats, from: CategoryStats): void {
  into.total += from.total
  into.measured += from.measured
  into.skipped_latex += from.skipped_latex
  into.skipped_empty_input += from.skipped_empty_input
  into.skipped_no_world += from.skipped_no_world
  into.skipped_no_unicode += from.skipped_no_unicode
  into.match += from.match
  into.mismatch += from.mismatch
  // 미스매치 상세는 합치지 않음 (파일 단위로 따로 보존)
}

function pct(num: number, denom: number): string {
  if (denom === 0) return '0.0%'
  return `${((num / denom) * 100).toFixed(2)}%`
}

function unicodeForms(entry: TestCaseEntry): string[] {
  if (entry.alternatives) return entry.alternatives.map(({ unicode }) => unicode)
  return entry.unicode ? [entry.unicode] : []
}

async function processFile(
  filePath: string,
  relPath: string,
): Promise<CategoryStats> {
  const raw = await readFile(filePath, 'utf-8')
  const entries: TestCaseEntry[] = JSON.parse(raw)
  const s = newStats()
  s.total = entries.length

  entries.forEach((entry, idx) => {
    const lineNumber = idx + 1
    if (entry.note === 'LaTeX') {
      s.skipped_latex++
      return
    }
    if (!entry.input || entry.input.trim() === '') {
      s.skipped_empty_input++
      return
    }
    if (!entry.world || entry.world === '') {
      s.skipped_no_world++
      return
    }
    const pdfForms = unicodeForms(entry)
    if (pdfForms.length === 0 || pdfForms.every((unicode) => unicode === '')) {
      s.skipped_no_unicode++
      return
    }

    s.measured++
    if (pdfForms.includes(entry.world)) {
      s.match++
    } else {
      s.mismatch++
      if (s.mismatches.length < 50) {
        s.mismatches.push({
          file: relPath,
          line: lineNumber,
          input: entry.input,
          pdf: pdfForms.join(' / '),
          world: entry.world,
        })
      }
    }
  })

  return s
}

async function main() {
  const perCategory = new Map<string, CategoryStats>()
  const perFile = new Map<string, CategoryStats>()
  const grand = newStats()

  const dirs = await readdir(TEST_CASES_DIR, { withFileTypes: true })
  for (const dirent of dirs) {
    if (!dirent.isDirectory()) continue
    const dir = dirent.name
    const dirPath = join(TEST_CASES_DIR, dir)
    const files = await readdir(dirPath)
    const jsonFiles = files.filter((f) => f.endsWith('.json')).sort()

    const catStats = newStats()

    for (const file of jsonFiles) {
      const filePath = join(dirPath, file)
      const relPath = `${dir}/${file}`
      const fileStats = await processFile(filePath, relPath)
      perFile.set(relPath, fileStats)
      add(catStats, fileStats)
    }

    perCategory.set(dir, catStats)
    add(grand, catStats)
  }

  // 보고서 생성
  await mkdir(dirname(REPORT_PATH), { recursive: true })

  const lines: string[] = []
  lines.push('# 점자세상 (braillekorea.org) 정답률 벤치마크')
  lines.push('')
  lines.push(`- 측정일: ${new Date().toISOString().slice(0, 10)}`)
  lines.push('- 비교 기준: PDF 규정 (2024 개정 한국 점자 규정)')
  lines.push('  - PDF 정답 = test_cases JSON 의 `unicode` 필드')
  lines.push('  - 점자세상 결과 = test_cases JSON 의 `world` 필드 (fetch-world.ts 가 braillekorea.org API 에서 수집)')
  lines.push('- 비교 방식: 단순 유니코드 문자열 동치 (`world === unicode`)')
  lines.push('- Skip 정책: LaTeX 변형, 빈 input, world 미수집, unicode 미정의 항목 제외')
  lines.push('')

  lines.push('## 전체 요약')
  lines.push('')
  lines.push('| 항목 | 값 |')
  lines.push('|---|---:|')
  lines.push(`| 전체 testcase | ${grand.total} |`)
  lines.push(`| 측정 대상 | ${grand.measured} |`)
  lines.push(`| 제외 (LaTeX) | ${grand.skipped_latex} |`)
  lines.push(`| 제외 (빈 input) | ${grand.skipped_empty_input} |`)
  lines.push(`| 제외 (world 미수집) | ${grand.skipped_no_world} |`)
  lines.push(`| 제외 (unicode 없음) | ${grand.skipped_no_unicode} |`)
  lines.push(`| **점자세상 PDF 정답 일치** | **${grand.match} (${pct(grand.match, grand.measured)})** |`)
  lines.push(`| **점자세상 PDF 정답 불일치** | **${grand.mismatch} (${pct(grand.mismatch, grand.measured)})** |`)
  lines.push('')
  lines.push('> 참고 — braillify 의 PDF 정답 일치: **2419/2419 = 100.00%** (cargo test test_by_testcase).')
  lines.push('> 단, braillify 측정에는 `KNOWN_FAILURES` 라우팅이 포함되어 있어 raw encode 정답률은 별도 측정 필요.')
  lines.push('')

  lines.push('## 카테고리별')
  lines.push('')
  lines.push('| 카테고리 | 전체 | 측정 | 일치 | 불일치 | 일치율 |')
  lines.push('|---|---:|---:|---:|---:|---:|')
  const catKeys = [...perCategory.keys()].sort()
  for (const k of catKeys) {
    const s = perCategory.get(k)
    if (!s) continue
    lines.push(`| ${k}/ | ${s.total} | ${s.measured} | ${s.match} | ${s.mismatch} | ${pct(s.match, s.measured)} |`)
  }
  lines.push('')

  lines.push('## 파일별 (상위 30개, 일치율 낮은 순)')
  lines.push('')
  const fileEntries = [...perFile.entries()]
    .filter(([, s]) => s.measured > 0)
    .sort((a, b) => a[1].match / a[1].measured - b[1].match / b[1].measured)
    .slice(0, 30)
  lines.push('| 파일 | 측정 | 일치 | 불일치 | 일치율 |')
  lines.push('|---|---:|---:|---:|---:|')
  for (const [k, s] of fileEntries) {
    lines.push(`| ${k} | ${s.measured} | ${s.match} | ${s.mismatch} | ${pct(s.match, s.measured)} |`)
  }
  lines.push('')

  lines.push('## 해석')
  lines.push('')
  lines.push('이 측정은 점자세상의 PDF 규정 준수도에 대한 객관적 지표이다.')
  lines.push('일치하지 않는 testcase는 점자세상 결과가 2024 개정 한국 점자 규정과 다르다는 의미이며,')
  lines.push('braillify 의 정답성과는 무관하다 (braillify 알고리즘은 점자세상 결과를 참조하지 않는다 — AGENTS.md RED LINE).')
  lines.push('')
  lines.push('상세 미스매치 목록은 [`WORLD_MISMATCHES.md`](./WORLD_MISMATCHES.md) 참고.')
  lines.push('')

  await writeFile(REPORT_PATH, lines.join('\n'), 'utf-8')

  // 미스매치 상세 보고서
  const mmLines: string[] = []
  mmLines.push('# 점자세상 미스매치 상세 (PDF 정답 ≠ world)')
  mmLines.push('')
  mmLines.push('각 카테고리에서 처음 50개까지만 기록한다 (보고서 크기 제한).')
  mmLines.push('')
  for (const [relPath, s] of perFile.entries()) {
    if (s.mismatches.length === 0) continue
    mmLines.push(`## ${relPath} (${s.mismatch} 미스매치)`)
    mmLines.push('')
    mmLines.push('| line | input | PDF (unicode) | 점자세상 (world) |')
    mmLines.push('|---:|---|---|---|')
    for (const m of s.mismatches) {
      const inEsc = m.input.replace(/\|/g, '\\|')
      mmLines.push(`| ${m.line} | \`${inEsc}\` | \`${m.pdf}\` | \`${m.world}\` |`)
    }
    mmLines.push('')
  }
  await writeFile(MISMATCH_PATH, mmLines.join('\n'), 'utf-8')

  // 표준 출력 요약
  console.log('=' .repeat(60))
  console.log('점자세상 정답률 벤치마크 결과')
  console.log('=' .repeat(60))
  console.log(`전체:     ${grand.total}`)
  console.log(`측정:     ${grand.measured}`)
  console.log(`일치:     ${grand.match} (${pct(grand.match, grand.measured)})`)
  console.log(`불일치:   ${grand.mismatch} (${pct(grand.mismatch, grand.measured)})`)
  console.log(`Skip:     ${grand.skipped_latex + grand.skipped_empty_input + grand.skipped_no_world + grand.skipped_no_unicode}`)
  console.log(`  - LaTeX:           ${grand.skipped_latex}`)
  console.log(`  - 빈 input:        ${grand.skipped_empty_input}`)
  console.log(`  - world 미수집:    ${grand.skipped_no_world}`)
  console.log(`  - unicode 없음:    ${grand.skipped_no_unicode}`)
  console.log('')
  console.log(`보고서: bench/WORLD_BENCH.md`)
  console.log(`미스매치: bench/WORLD_MISMATCHES.md`)
}

main().catch((err) => {
  console.error(err)
  process.exit(1)
})
