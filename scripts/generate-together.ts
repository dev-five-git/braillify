import { readFile } from 'node:fs/promises'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

import { validatePartners } from './together-data'
import { generateTogetherImages, pruneTogetherImages } from './together-images'
import { publishPartners, validatePartnerLogos } from './together-publish'

const landing = resolve(dirname(fileURLToPath(import.meta.url)), '../apps/landing')
const partners = validatePartners(JSON.parse(await readFile(join(landing, 'together.json'), 'utf8')))
const publicDir = join(landing, 'public')
await validatePartnerLogos(partners, publicDir)
await generateTogetherImages(partners, publicDir)
await pruneTogetherImages(partners, publicDir)
await publishPartners(partners, publicDir)
