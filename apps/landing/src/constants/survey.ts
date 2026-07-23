export const SURVEY_URL =
  'https://docs.google.com/forms/d/e/1FAIpQLSdXdiajD92xptfYMhFT9Xsu2hWgrCA21DMSnUlCUsB-X0lZLw/viewform'

/**
 * 설문 노출 마감 시각 (KST 기준).
 * 2026-06-29 23:59:59 KST 까지 노출하고, 2026-06-30 00:00:00 KST 부터는 숨긴다.
 * (= 2026-06-29T15:00:00Z UTC)
 */
export const SURVEY_END = new Date('2026-06-30T00:00:00+09:00').getTime()

/** 현재 시각이 설문 노출 기간 내인지 여부. */
export function isSurveyActive(now: number = Date.now()): boolean {
  return now < SURVEY_END
}
