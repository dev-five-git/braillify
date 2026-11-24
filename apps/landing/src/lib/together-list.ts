export type Partner = {
  issue: number
  kind: string
  name: string
  logo: string
}

export async function loadTogetherList(
  signal: AbortSignal,
  request: (
    input: RequestInfo | URL,
    init?: RequestInit,
  ) => Promise<Response> = fetch,
  timeoutMs = 3000,
): Promise<Partner[]> {
  const controller = new AbortController()
  const abort = () => controller.abort()
  signal.addEventListener('abort', abort, { once: true })
  if (signal.aborted) abort()
  const timer = setTimeout(abort, timeoutMs)
  try {
    const cancelled = new Promise<never>((_, reject) => {
      if (controller.signal.aborted) reject(new Error('List request cancelled'))
      else
        controller.signal.addEventListener(
          'abort',
          () => reject(new Error('List request cancelled')),
          { once: true },
        )
    })
    return await Promise.race([
      cancelled,
      (async () => {
        const response = await request('/together.json', {
          signal: controller.signal,
        })
        if (!response.ok) throw new Error('List request failed')
        const data: unknown = await response.json()
        if (!Array.isArray(data)) throw new Error('Invalid list')
        const seen = new Set<number>()
        for (const item of data) {
          if (
            !item ||
            typeof item !== 'object' ||
            !Number.isSafeInteger(item.issue) ||
            item.issue <= 0 ||
            seen.has(item.issue) ||
            !['기업', '기관', '학교', '단체'].includes(item.kind) ||
            typeof item.name !== 'string' ||
            !item.name.trim() ||
            typeof item.logo !== 'string' ||
            !/^\/images\/together\/asset-[a-zA-Z0-9_-]+\.(png|jpg|jpeg)$/.test(
              item.logo,
            )
          )
            throw new Error('Invalid list entry')
          seen.add(item.issue)
        }
        return data as Partner[]
      })(),
    ])
  } finally {
    clearTimeout(timer)
    signal.removeEventListener('abort', abort)
  }
}
