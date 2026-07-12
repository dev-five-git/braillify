/**
 * Fetch braille conversion results from 점자세상 (braillekorea.org) and add
 * `world` field to each test case entry.
 *
 * 새 API (2026-05): `/braille/brailleProcAjax.do` (POST, sourceText=, X-CSRF-TOKEN
 * 헤더 + JSESSIONID 쿠키 필요). CSRF 토큰은 메인 페이지 HTML 의
 * `<meta name="_csrf" content="UUID">` 에 들어있고 세션 만료 시 갱신 필요.
 *
 * 정책:
 *  - 성공 시에만 `entry.world` 를 새 값으로 갱신한다.
 *  - skip 항목 (LaTeX 변형, 빈 input) 은 기존 값을 **보존**한다 (덮어쓰지 않음).
 *  - 실패 (HTTP 에러, resultCode≠0, 네트워크 에러) 시에도 기존 값을 **보존**한다.
 *    → 이전 실행에서 성공한 결과가 일시 장애로 사라지는 일을 방지.
 *
 * 병렬화:
 *  - 파일 단위 순차 처리 (부분 진행 상황 즉시 write 로 보존).
 *  - 파일 내 entry 들은 `CONCURRENCY` 개씩 동시 fetch (Promise.allSettled).
 *  - 각 batch 사이에 짧은 delay 로 서버 부하 완화.
 *  - 동시 요청들은 하나의 세션 (CSRF + JSESSIONID) 을 공유한다.
 *  - 403/401 발생 시 mutex 로 보호된 1회 재bootstrap 후 재시도.
 *
 * Usage: bun run scripts/fetch-world.ts
 */

import { readdir, readFile, writeFile } from 'node:fs/promises'
import { join } from 'node:path'

const BOOTSTRAP_URL =
  'https://www.braillekorea.org/menu/120/program/303/braille.do'
const API_URL = 'https://www.braillekorea.org/braille/brailleProcAjax.do'
/** 파일 내 동시 요청 수 (점자세상 서버 부하와 안정성의 절충점). */
const CONCURRENCY = 8
/** 각 batch 종료 후 대기 시간 (ms). 0 이면 batch 간 지연 없음. */
const BATCH_DELAY_MS = 50
const TEST_CASES_DIR = join(import.meta.dirname, '..', 'test_cases')

interface TestCaseEntry {
  input: string
  internal?: string
  expected?: string
  unicode?: string
  alternatives?: TestAlternative[]
  note?: string
  world?: string
}

interface TestAlternative {
  internal: string
  expected: string
  unicode: string
}

interface Session {
  cookies: string
  csrfToken: string
}

interface SessionRef {
  current: Session
}

interface BrailleResponse {
  sourceText?: string | null
  ascii?: string
  braille?: string
  usageCount?: number
  resultCode?: number
}

async function bootstrap(): Promise<Session> {
  const res = await fetch(BOOTSTRAP_URL, {
    headers: { 'Accept-Language': 'ko-KR,ko;q=0.9,en;q=0.8' },
  })
  if (!res.ok) {
    throw new Error(`bootstrap failed: ${res.status}`)
  }
  const setCookies = (res.headers.getSetCookie() || [])
    .map((c) => c.split(';')[0])
    .filter((c) => c.length > 0)
  if (setCookies.length === 0) {
    throw new Error('bootstrap returned no Set-Cookie header')
  }
  const html = await res.text()
  const match = html.match(/<meta name="_csrf" content="([^"]+)"/)
  if (!match) {
    throw new Error('CSRF token not found in bootstrap response')
  }
  return {
    cookies: setCookies.join('; '),
    csrfToken: match[1],
  }
}

/** Mutex-guarded session refresh: 동시 호출들은 같은 bootstrap Promise 공유. */
let bootstrapInFlight: Promise<Session> | null = null
async function refreshSession(ref: SessionRef): Promise<void> {
  if (!bootstrapInFlight) {
    bootstrapInFlight = bootstrap().finally(() => {
      bootstrapInFlight = null
    })
  }
  ref.current = await bootstrapInFlight
}

class SessionExpired extends Error {
  constructor(status: number) {
    super(`unauthorized (${status}) - session expired`)
  }
}

async function fetchBrailleOnce(
  input: string,
  session: Session,
): Promise<string> {
  const body = `sourceText=${encodeURIComponent(input)}`
  const res = await fetch(API_URL, {
    method: 'POST',
    headers: {
      Accept: 'application/json, text/javascript, */*; q=0.01',
      'Accept-Language': 'ko-KR,ko;q=0.9,en;q=0.8',
      'Content-Type': 'application/x-www-form-urlencoded; charset=UTF-8',
      'X-Requested-With': 'XMLHttpRequest',
      'X-CSRF-TOKEN': session.csrfToken,
      Cookie: session.cookies,
      Referer: BOOTSTRAP_URL,
    },
    body,
  })
  if (res.status === 403 || res.status === 401) {
    throw new SessionExpired(res.status)
  }
  if (!res.ok) {
    throw new Error(`API returned ${res.status}`)
  }
  const data = (await res.json()) as BrailleResponse
  if (typeof data.resultCode === 'number' && data.resultCode !== 0) {
    throw new Error(`API resultCode=${data.resultCode}`)
  }
  if (typeof data.braille !== 'string') {
    throw new Error('API response missing braille field')
  }
  return data.braille
}

async function fetchWithRetry(
  input: string,
  ref: SessionRef,
): Promise<string> {
  try {
    return await fetchBrailleOnce(input, ref.current)
  } catch (err) {
    if (err instanceof SessionExpired) {
      await refreshSession(ref)
      return await fetchBrailleOnce(input, ref.current)
    }
    throw err
  }
}

function shouldSkip(entry: TestCaseEntry): boolean {
  if (entry.note === 'LaTeX') return true
  if (!entry.input || entry.input.trim() === '') return true
  return false
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

interface FileStats {
  total: number
  fetched: number
  skipped: number
  preserved: number
  errors: number
}

async function processFile(
  filePath: string,
  ref: SessionRef,
): Promise<FileStats> {
  const raw = await readFile(filePath, 'utf-8')
  const entries: TestCaseEntry[] = JSON.parse(raw)
  const stats: FileStats = {
    total: entries.length,
    fetched: 0,
    skipped: 0,
    preserved: 0,
    errors: 0,
  }

  // 처리할 entry index 만 수집. skip 항목은 기존 값 보존 (덮어쓰지 않음).
  const tasks: Array<{ idx: number; input: string }> = []
  for (let i = 0; i < entries.length; i++) {
    const e = entries[i]
    if (shouldSkip(e)) {
      stats.skipped++
      continue
    }
    tasks.push({ idx: i, input: e.input })
  }

  // CONCURRENCY 개씩 batch.
  for (let i = 0; i < tasks.length; i += CONCURRENCY) {
    const chunk = tasks.slice(i, i + CONCURRENCY)
    const results = await Promise.allSettled(
      chunk.map((t) => fetchWithRetry(t.input, ref)),
    )
    for (let j = 0; j < chunk.length; j++) {
      const r = results[j]
      const idx = chunk[j].idx
      if (r.status === 'fulfilled') {
        entries[idx].world = r.value
        stats.fetched++
      } else {
        // 실패 시 기존 entry.world 보존. 이전에 성공한 결과가
        // 일시 장애로 손실되는 것을 방지.
        if (entries[idx].world && entries[idx].world !== '') {
          stats.preserved++
        }
        stats.errors++
        const reason = (r.reason as Error).message
        console.error(
          `  Error for "${chunk[j].input.slice(0, 30)}...": ${reason}`,
        )
      }
    }
    if (BATCH_DELAY_MS > 0 && i + CONCURRENCY < tasks.length) {
      await sleep(BATCH_DELAY_MS)
    }
  }

  await writeFile(filePath, `${JSON.stringify(entries, null, 2)}\n`, 'utf-8')
  return stats
}

async function main(): Promise<void> {
  console.log('Bootstrapping session ...')
  const ref: SessionRef = { current: await bootstrap() }
  console.log(
    `  JSESSIONID acquired, CSRF token = ${ref.current.csrfToken.slice(0, 8)}...`,
  )
  console.log(`  Concurrency: ${CONCURRENCY}, batch delay: ${BATCH_DELAY_MS}ms`)
  console.log(
    `  Policy: 성공 시에만 갱신 / skip·실패 시 기존 world 값 보존`,
  )

  const dirs = await readdir(TEST_CASES_DIR)
  const grand: FileStats = {
    total: 0,
    fetched: 0,
    skipped: 0,
    preserved: 0,
    errors: 0,
  }

  for (const dir of dirs) {
    const dirPath = join(TEST_CASES_DIR, dir)
    let files: string[]
    try {
      files = await readdir(dirPath)
    } catch {
      continue
    }
    const jsonFiles = files.filter((f) => f.endsWith('.json')).sort()
    if (jsonFiles.length === 0) continue

    console.log(`\n📁 ${dir}/`)

    for (const file of jsonFiles) {
      const filePath = join(dirPath, file)
      process.stdout.write(`  ${file} ... `)
      const start = performance.now()
      const stats = await processFile(filePath, ref)
      const dur = ((performance.now() - start) / 1000).toFixed(1)
      console.log(
        `✓ ${stats.fetched} fetched, ${stats.skipped} skipped, ${stats.errors} errors${stats.preserved > 0 ? `, ${stats.preserved} preserved` : ''} (${stats.total} total) [${dur}s]`,
      )
      grand.total += stats.total
      grand.fetched += stats.fetched
      grand.skipped += stats.skipped
      grand.preserved += stats.preserved
      grand.errors += stats.errors
    }
  }

  console.log(`\n${'='.repeat(60)}`)
  console.log(`Total: ${grand.total} entries`)
  console.log(
    `Fetched: ${grand.fetched} | Skipped: ${grand.skipped} | Errors: ${grand.errors} | Preserved: ${grand.preserved}`,
  )
  console.log(`${'='.repeat(60)}`)
}

main().catch((err) => {
  console.error(err)
  process.exit(1)
})
