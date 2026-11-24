import { readFile } from 'node:fs/promises'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

import { validatePartners } from './together-data'
import { publishPartners } from './together-publish'

const landing = resolve(dirname(fileURLToPath(import.meta.url)), '../apps/landing')
const partners = validatePartners(JSON.parse(await readFile(join(landing, 'together.json'), 'utf8')))
await publishPartners(partners, join(landing, 'public'))
