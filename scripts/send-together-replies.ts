import { readFile } from 'node:fs/promises'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

import { validatePartners } from './together-data'
import { sendApprovalReplies } from './together-replies'

const path = resolve(dirname(fileURLToPath(import.meta.url)), '../apps/landing/together.json')
const partners = validatePartners(JSON.parse(await readFile(path, 'utf8')))
await sendApprovalReplies(partners, process.env.GITHUB_TOKEN || '')
