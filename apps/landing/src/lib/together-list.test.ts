import { expect, spyOn, test } from 'bun:test'

import { loadTogetherList } from './together-list'

test('기본 요청으로 공개 목록을 가져온다', async () => {
  const request = spyOn(globalThis, 'fetch').mockResolvedValue(
    Response.json([]),
  )
  try {
    expect(await loadTogetherList(new AbortController().signal)).toEqual([])
    expect(request).toHaveBeenCalledWith('/together.json', {
      signal: expect.any(AbortSignal),
    })
  } finally {
    request.mockRestore()
  }
})

test('빈 목록과 승인된 목록을 반환한다', async () => {
  expect(
    await loadTogetherList(
      new AbortController().signal,
      async () => Response.json([]),
      50,
    ),
  ).toEqual([])
  const item = {
    issue: 1,
    kind: '학교',
    name: '학교',
    logo: '/images/together/asset-1.png',
  }
  expect(
    await loadTogetherList(
      new AbortController().signal,
      async () => Response.json([item]),
      50,
    ),
  ).toEqual([item])
})

test('실패, 잘못된 응답 및 시간 초과는 오류로 처리한다', async () => {
  await expect(
    loadTogetherList(
      new AbortController().signal,
      async () => new Response('', { status: 500 }),
      50,
    ),
  ).rejects.toThrow()
  await expect(
    loadTogetherList(
      new AbortController().signal,
      async () => Response.json([{ issue: 1 }]),
      50,
    ),
  ).rejects.toThrow()
  await expect(
    loadTogetherList(
      new AbortController().signal,
      async () => Response.json({}),
      50,
    ),
  ).rejects.toThrow()
  await expect(
    loadTogetherList(
      new AbortController().signal,
      () => new Promise(() => {}),
      10,
    ),
  ).rejects.toThrow()
})

test('시간 초과 후 다시 시도하면 새 목록을 읽는다', async () => {
  let calls = 0
  const request = async () => {
    calls++
    if (calls === 1) return await new Promise<Response>(() => {})
    return Response.json([])
  }
  await expect(
    loadTogetherList(new AbortController().signal, request, 10),
  ).rejects.toThrow()
  expect(
    await loadTogetherList(new AbortController().signal, request, 50),
  ).toEqual([])
})

test('화면을 떠나면 진행 중인 요청을 취소한다', async () => {
  const controller = new AbortController()
  let requestSignal: AbortSignal | null | undefined
  const pending = loadTogetherList(
    controller.signal,
    (_input, init) => {
      requestSignal = init?.signal
      return new Promise(() => {})
    },
    1000,
  )
  controller.abort()
  await expect(pending).rejects.toThrow()
  expect(requestSignal?.aborted).toBe(true)
})

test('이미 취소된 요청은 데이터를 반환하지 않는다', async () => {
  const controller = new AbortController()
  controller.abort()
  await expect(
    loadTogetherList(controller.signal, async () => Response.json([]), 50),
  ).rejects.toThrow()
})
