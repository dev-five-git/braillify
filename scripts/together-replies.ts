import type { Partner } from './together-data'

const origin = 'https://braillify.kr'
const html = (text: string) => text.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;')
const markdown = (text: string) => text.replace(/([\\`*_{}\[\]()#+.!|>-])/g, '\\$1')

export function publishedPartner(data: unknown, partner: Partner) {
  return Array.isArray(data) && data.some((item) => item && Object.keys(partner).every((key) => item[key] === partner[key as keyof Partner]))
}

export function approvalMessage(partner: Partner) {
  const id = `org-${partner.issue}`
  const card = `${origin}/together#${id}`
  const cert = `${origin}/together/cert/${id}`
  const badge = `${origin}/together/badge/${id}`
  const [year, month] = partner.approvedAt.split('-')
  const alt = `Braillify와 함께하는 ${partner.kind} · ${year}년 ${Number(month)}월 승인`
  const embed = (color: string) => `<a href="${card}"><img src="${badge}-${color}.png" alt="${html(alt)}" width="300" height="80"></a>`
  return [
    `<!-- braillify-together-approval:${partner.issue} -->`,
    '승인됐어요. Braillify와 함께해 주셔서 고마워요.',
    `${markdown(partner.name)}를 함께하는 곳에 올렸어요.`,
    card,
    `[인증 이미지 PNG 내려받기 (1200×630)](${cert}.png) · [2배 크기 (2400×1260)](${cert}@2x.png)`,
    '누리집 배지 (밝은 바탕용 · 어두운 바탕용):',
    `[밝은 배지](${badge}-light.png) · [2배](${badge}-light@2x.png) · [어두운 배지](${badge}-dark.png) · [2배](${badge}-dark@2x.png)`,
    '```html', embed('light'), '```', '```html', embed('dark'), '```',
    '게시를 멈추고 싶으면 contact@devfive.kr로 알려 주세요.',
  ].join('\n\n')
}

export async function sendApprovalReplies(partners: Partner[], token: string, request: typeof fetch = fetch) {
  if (!token) throw new Error('GITHUB_TOKEN is required')
  if (!partners.length) return
  const listing = await request(`${origin}/together.json`, { redirect: 'error', cache: 'no-store' })
  if (!listing.ok) throw new Error('Published listing unavailable')
  const data: unknown = await listing.json()
  for (const partner of partners) {
    if (!publishedPartner(data, partner)) throw new Error(`Published listing differs for issue ${partner.issue}`)
    const id = `org-${partner.issue}`
    const paths = [`cert/${id}.png`, `cert/${id}@2x.png`, ...['light', 'dark'].flatMap((color) => [`badge/${id}-${color}.png`, `badge/${id}-${color}@2x.png`])]
    for (const path of paths) {
      const response = await request(`${origin}/together/${path}`, { redirect: 'error', cache: 'no-store' })
      if (!response.ok || !(response.headers.get('content-type') || '').toLowerCase().startsWith('image/png')) throw new Error(`Published image unavailable: ${path}`)
      const bytes = new Uint8Array(await response.arrayBuffer())
      if (bytes.length < 8 || ![137, 80, 78, 71, 13, 10, 26, 10].every((byte, i) => bytes[i] === byte)) throw new Error(`Invalid published image: ${path}`)
    }
  }
  const headers = { Authorization: `Bearer ${token}`, Accept: 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28' }
  for (const partner of partners) {
    const endpoint = `https://api.github.com/repos/dev-five-git/braillify/issues/${partner.issue}/comments`
    const marker = `<!-- braillify-together-approval:${partner.issue} -->`
    let alreadySent = false
    for (let page = 1; ; page++) {
      const response = await request(`${endpoint}?per_page=100&page=${page}`, { headers, redirect: 'error' })
      if (!response.ok) throw new Error(`Unable to read issue ${partner.issue} comments`)
      const comments: Array<{ body?: string; user?: { login?: string } }> = await response.json()
      if (!Array.isArray(comments)) throw new Error('Invalid GitHub response')
      if (comments.some((comment) => comment.user?.login === 'github-actions[bot]' && comment.body?.includes(marker))) alreadySent = true
      if (alreadySent || comments.length < 100) break
    }
    if (!alreadySent) {
      const response = await request(endpoint, { method: 'POST', headers: { ...headers, 'Content-Type': 'application/json' }, body: JSON.stringify({ body: approvalMessage(partner) }), redirect: 'error' })
      if (!response.ok) throw new Error(`Unable to reply to issue ${partner.issue}`)
    }
  }
}
