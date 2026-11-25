import { readFile, readdir, realpath, stat, writeFile } from 'node:fs/promises'
import { join, sep } from 'node:path'

import type { Partner } from './together-data'

export async function validatePartnerLogos(partners: Partner[], root: string) {
  const publicDir = await realpath(root)
  const logoDir = join(publicDir, 'images', 'together')
  let names: string[]
  try {
    const resolved = await realpath(logoDir)
    if (!resolved.startsWith(`${publicDir}${sep}`)) throw new Error('Logo directory is outside public directory')
    names = await readdir(resolved)
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error
    names = []
  }
  const referenced = new Set(partners.map((partner) => partner.logo.slice('/images/together/'.length)))
  for (const name of names) {
    if (/^asset-[a-zA-Z0-9_-]+\.(png|jpg|jpeg)$/i.test(name) && !referenced.has(name)) throw new Error(`Unreferenced logo must be removed: ${name}`)
  }
  for (const partner of partners) {
    const filename = await realpath(join(publicDir, partner.logo.slice(1)))
    if (!filename.startsWith(`${publicDir}${sep}`)) throw new Error(`Logo is outside public directory: ${partner.logo}`)
    const info = await stat(filename)
    if (!info.isFile() || info.size > 2 * 1024 * 1024) throw new Error(`Invalid logo size: ${partner.logo}`)
    const bytes = await readFile(filename)
    const png = bytes.subarray(0, 8).equals(Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]))
    const jpeg = bytes[0] === 0xff && bytes[1] === 0xd8 && bytes[2] === 0xff
    if (partner.logo.endsWith('.png') ? !png : !jpeg) throw new Error(`Invalid logo format: ${partner.logo}`)
  }
}

export async function publishPartners(partners: Partner[], root: string) {
  await validatePartnerLogos(partners, root)
  await writeFile(join(root, 'together.json'), `${JSON.stringify(partners)}\n`)
}
