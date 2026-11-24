import { mkdtemp, readdir, realpath, rm, unlink } from 'node:fs/promises'
import { join } from 'node:path'
import { spawnSync } from 'node:child_process'

import type { Partner } from './together-data'

export async function withChromeProfile<T>(work: string, run: (profile: string) => T | Promise<T>): Promise<T> {
  const profile = await mkdtemp(join(work, 'profile-'))
  try { return await run(profile) } finally { await rm(profile, { recursive: true, force: true }) }
}

export const escapeHtml = (value: string) => value.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;')

export function translateName(executable: string, name: string) {
  const result = spawnSync(executable, ['--', name], { encoding: 'utf8' })
  if (result.status !== 0) throw new Error(`Braille generation failed: ${result.stderr || result.error}`)
  return result.stdout
}

export function brailleSvg(text: string) {
  const cells = Array.from(text)
  const scale = Math.min(1, 960 / (cells.length * 22 || 1))
  const radius = Math.max(2, 3 * scale)
  const width = cells.length ? (cells.length - 1) * 22 * scale + 9 * scale + 2 * radius : 0
  if (width > 960) throw new Error('Braille is too long to fit')
  const height = (cells.some((cell) => ((cell.codePointAt(0)! - 0x2800) & 0xc0) !== 0) ? 27 : 18) * scale + 2 * radius
  const circles = cells.flatMap((cell, index) => {
    const bits = cell.codePointAt(0)! - 0x2800
    if (bits < 0 || bits > 255) throw new Error('Invalid braille cell')
    return [0, 1, 2, 3, 4, 5, 6, 7].filter((bit) => bits & (1 << bit)).map((bit) => {
      const x = index * 22 * scale + radius + (bit < 3 || bit === 6 ? 0 : 9 * scale)
      const y = radius + [0, 9, 18, 0, 9, 18, 27, 27][bit] * scale
      return `<circle cx="${x}" cy="${y}" r="${radius}"/>`
    })
  }).join('')
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}" fill="#787878">${circles}</svg>`
}

export async function pruneTogetherImages(partners: Partner[], publicDir: string) {
  const safeRoot = await realpath(publicDir)
  const approved = new Set(partners.map((partner) => String(partner.issue)))
  for (const group of ['cert', 'badge']) {
    const directory = join(safeRoot, 'together', group)
    let resolved: string
    try { resolved = await realpath(directory) } catch (error) { if ((error as NodeJS.ErrnoException).code === 'ENOENT') continue; throw error }
    if (!resolved.startsWith(`${safeRoot}${process.platform === 'win32' ? '\\' : '/'}`)) throw new Error('Image directory is outside public directory')
    for (const entry of await readdir(resolved, { withFileTypes: true })) {
      const match = group === 'cert' ? /^org-(\d+)(?:@2x)?\.png$/.exec(entry.name) : /^org-(\d+)-(?:light|dark)(?:@2x)?\.png$/.exec(entry.name)
      if (entry.isFile() && match && !approved.has(match[1])) await unlink(join(resolved, entry.name))
    }
  }
}
