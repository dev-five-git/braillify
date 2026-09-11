/**
 * 점사랑 점역 결과(`jeomsarang` 필드)를 BrailleTransLibrary DLL 로 직접 수집한다.
 *
 * 이전에는 fetch-jeomsarang.py 가 점사랑 7.0 GUI 를 자동화했으나, 배포된
 * BrailleTransLibrary-*.zip 을 쓰면 GUI 없이 같은 엔진으로 점역할 수 있다.
 *
 * 동작:
 *  1. 저장소 루트의 BrailleTransLibrary-*.zip 을 작업 폴더에 푼다.
 *  2. scripts/jeomsarang-lib/btl.cs 를 x86 으로 컴파일한다 (.NET Framework csc).
 *  3. test_cases 의 각 JSON 을 순회하며 `jeomsarang` 을 채운다.
 *
 * DLL 이 32비트이므로 Windows + .NET Framework csc.exe 가 필요하다.
 *
 * Usage:
 *   bun run scripts/fetch-jeomsarang-lib.ts
 *   FETCH_JEOMSARANG_DIR=korean,math bun run scripts/fetch-jeomsarang-lib.ts
 */

import { spawnSync } from 'node:child_process'
import {
  appendFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from 'node:fs'
import { join } from 'node:path'

const ROOT = join(import.meta.dirname, '..')
const TEST_CASES_DIRECTORY = join(ROOT, 'test_cases')
const WORK_DIRECTORY = join(ROOT, 'target', 'jeomsarang-lib')
const LIBRARY_DIRECTORY = join(WORK_DIRECTORY, 'BrailleTransLibrary')
const CSC =
  'C:\\Windows\\Microsoft.NET\\Framework\\v4.0.30319\\csc.exe'
const GRADE_MODE = process.env.JEOMSARANG_GRADE ?? '0'
const RULE_VERSION = process.env.JEOMSARANG_RULE_VERSION ?? '1'

interface TestCaseEntry {
  input: string
  note?: string
  jeomsarang?: string
}

function prepareLibrary(): string {
  const archive = readdirSync(ROOT).find((name) =>
    /^BrailleTransLibrary.*\.zip$/.test(name),
  )
  if (!archive) {
    throw new Error('BrailleTransLibrary-*.zip not found in repository root')
  }
  if (!existsSync(LIBRARY_DIRECTORY)) {
    mkdirSync(WORK_DIRECTORY, { recursive: true })
    const unzip = spawnSync(
      'powershell',
      [
        '-NoProfile',
        '-Command',
        `Expand-Archive -LiteralPath '${join(ROOT, archive)}' -DestinationPath '${WORK_DIRECTORY}' -Force`,
      ],
      { encoding: 'utf8' },
    )
    if (unzip.status !== 0) {
      throw new Error(`failed to extract ${archive}: ${unzip.stderr}`)
    }
  }
  const executable = join(LIBRARY_DIRECTORY, 'btl.exe')
  const compile = spawnSync(
    CSC,
    [
      '/nologo',
      '/platform:x86',
      `/out:${executable}`,
      join(ROOT, 'scripts', 'jeomsarang-lib', 'btl.cs'),
    ],
    { encoding: 'utf8' },
  )
  if (compile.status !== 0) {
    throw new Error(`csc failed: ${compile.stdout}${compile.stderr}`)
  }
  return executable
}

function readLines(path: string): string[] {
  const lines = readFileSync(path, 'utf8').split('\n')
  if (lines[lines.length - 1] === '') lines.pop()
  return lines
}

/**
 * DLL 이 특정 입력(예: BMP 밖 수학 문자)에서 네이티브 크래시를 내면 프로세스가
 * 통째로 죽는다. 크래시한 줄만 빈 결과로 남기고 다음 줄부터 재개해, 한 건 때문에
 * 나머지 배치 전체를 잃지 않도록 한다.
 */
function translate(
  executable: string,
  inputs: string[],
): { results: string[]; crashed: number[] } {
  const inPath = join(LIBRARY_DIRECTORY, 'batch_in.txt')
  const outPath = join(LIBRARY_DIRECTORY, 'batch_out.txt')
  const escaped = inputs.map((input) =>
    input.replaceAll('\\', '\\\\').replaceAll('\n', '\\n'),
  )
  writeFileSync(inPath, `${escaped.join('\n')}\n`, 'utf8')
  writeFileSync(outPath, '', 'utf8')

  const crashed: number[] = []
  let start = 0
  while (start < inputs.length) {
    spawnSync(
      executable,
      [inPath, outPath, GRADE_MODE, RULE_VERSION, String(start)],
      { cwd: LIBRARY_DIRECTORY, encoding: 'utf8', maxBuffer: 1 << 26 },
    )
    const produced = readLines(outPath).length
    if (produced >= inputs.length) break
    crashed.push(produced)
    appendFileSync(outPath, '0\t\n', 'utf8')
    start = produced + 1
  }

  const results = readLines(outPath).map((line) =>
    line.replace(/\r$/, '').split('\t').slice(1).join('\t').replaceAll('\\n', '\n'),
  )
  while (results.length < inputs.length) results.push('')
  return { results, crashed }
}

function main(): void {
  const executable = prepareLibrary()
  const requested = process.env.FETCH_JEOMSARANG_DIR?.split(',')
  const directories = readdirSync(TEST_CASES_DIRECTORY, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .filter((name) => !requested || requested.includes(name))
    .sort()

  for (const directory of directories) {
    const base = join(TEST_CASES_DIRECTORY, directory)
    const files = readdirSync(base)
      .filter((file) => file.endsWith('.json'))
      .sort()
    let translated = 0
    let skipped = 0
    let crashes = 0
    for (const file of files) {
      const entries = JSON.parse(
        readFileSync(join(base, file), 'utf8'),
      ) as TestCaseEntry[]
      const indices: number[] = []
      for (let i = 0; i < entries.length; i++) {
        const entry = entries[i]
        if (entry.note === 'LaTeX' || !entry.input?.trim()) {
          skipped++
          continue
        }
        indices.push(i)
      }
      if (indices.length === 0) continue
      const { results, crashed } = translate(
        executable,
        indices.map((i) => entries[i].input),
      )
      for (const at of crashed) {
        console.log(
          `  crash: ${directory}/${file} #${indices[at]} ${JSON.stringify(entries[indices[at]].input).slice(0, 60)}`,
        )
      }
      crashes += crashed.length
      for (let k = 0; k < indices.length; k++) {
        entries[indices[k]].jeomsarang = results[k]
        if (results[k]) translated++
      }
      writeFileSync(
        join(base, file),
        `${JSON.stringify(entries, null, 2)}\n`,
        'utf8',
      )
    }
    console.log(
      `${directory}: ${translated} translated, ${skipped} skipped (LaTeX/empty), ${crashes} crashed`,
    )
  }
}

main()
