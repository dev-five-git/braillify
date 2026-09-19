import { readFile } from 'node:fs/promises'

import type { TestStatusReportManifest } from '@/types'

/**
 * Page counts for the result files emitted by `cargo test test_by_testcase`,
 * which `bun run build:landing` runs before `next build`.
 *
 * Only the manifest is read at build time; the pages themselves stay in
 * `public/` and are fetched by the browser one page at a time.
 */
export async function readReportManifest(): Promise<TestStatusReportManifest> {
  return JSON.parse(
    await readFile('public/test-status/manifest.json', 'utf-8'),
  ) as TestStatusReportManifest
}
