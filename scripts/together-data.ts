export interface Partner {
  issue: number
  kind: '기업' | '기관' | '학교' | '단체'
  name: string
  logo: string
  approvedAt: string
}

// Only maintainer-reviewed records belong in this list; an issue is not approval.
export function validatePartners(input: unknown): Partner[] {
  if (!Array.isArray(input)) throw new Error('Partner data must be an array')
  const seen = new Set<number>()
  return input.map((entry: unknown) => {
    if (!entry || typeof entry !== 'object' || Array.isArray(entry)) throw new Error('Invalid partner')
    const { issue, kind, name, logo, approvedAt } = entry as Record<string, unknown>
    if (!Number.isSafeInteger(issue) || (issue as number) <= 0 || seen.has(issue as number)) throw new Error('Invalid or duplicate issue')
    if (typeof kind !== 'string' || !['기업', '기관', '학교', '단체'].includes(kind)) throw new Error('Invalid kind')
    if (typeof name !== 'string' || !name.trim() || Array.from(name).length > 20 || /[\r\n\u0000-\u001f]/.test(name)) throw new Error('Invalid name')
    if (typeof logo !== 'string' || !/^\/images\/together\/asset-[a-zA-Z0-9_-]+\.(png|jpg|jpeg)$/.test(logo)) throw new Error('Invalid logo')
    if (typeof approvedAt !== 'string' || !/^\d{4}-\d{2}-\d{2}$/.test(approvedAt)) throw new Error('Invalid approval date')
    const date = new Date(`${approvedAt}T00:00:00Z`)
    if (Number.isNaN(date.valueOf()) || date.toISOString().slice(0, 10) !== approvedAt) throw new Error('Invalid approval date')
    seen.add(issue as number)
    return { issue, kind, name, logo, approvedAt } as Partner
  })
}
