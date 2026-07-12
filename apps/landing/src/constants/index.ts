import type { TestCaseFilter } from '@/components/test-case/TestCaseProvider'

export const TEST_CASE_FILTERS: { label: string; value: TestCaseFilter }[] = [
  { label: '한글', value: 'korean' },
  { label: '수학', value: 'math' },
  {
    label: '과학',
    value: 'science',
  },
  {
    label: '음악',
    value: 'music',
  },
  {
    label: '영어 표준 점자',
    value: 'english',
  },
  {
    label: '외국어',
    value: 'foreign-language',
  },
  {
    label: '국제음성기호',
    value: 'ipa',
  },
  {
    label: '말뭉치',
    value: 'corpus',
  },
]

export const TEST_CASE_FILTERS_MAP = Object.fromEntries(
  TEST_CASE_FILTERS.map((filter) => [filter.value, filter.label]),
) as Record<TestCaseFilter, string>

export const CATEGORY_PREFIX_MAP: Record<string, TestCaseFilter> = {
  'korean/': 'korean',
  'math/': 'math',
  'science/': 'science',
  'music/': 'music',
  'english/': 'english',
  'foreign-language/': 'foreign-language',
  'ipa/': 'ipa',
  'corpus/': 'corpus',
}

/**
 * Create a filter map based on rule_map.json keys.
 * Automatically classifies rules by key prefix (e.g. "korean/rule_1" → korean, "math/math_1" → math).
 * @param ruleMapKeys - Array of rule keys from rule_map.json
 * @returns Filter map grouped by categories
 */
export function createFilterMap(
  ruleMapKeys: string[],
): Record<TestCaseFilter, string[]> {
  const map: Record<TestCaseFilter, string[]> = {
    korean: [],
    math: [],
    science: [],
    music: [],
    english: [],
    'foreign-language': [],
    ipa: [],
    corpus: [],
  }

  for (const key of ruleMapKeys) {
    let matched = false
    for (const [prefix, category] of Object.entries(CATEGORY_PREFIX_MAP)) {
      if (key.startsWith(prefix)) {
        map[category].push(key)
        matched = true
        break
      }
    }
    if (!matched) {
      map.korean.push(key)
    }
  }

  return map
}

// Default FILTER_MAP for backward compatibility (legacy migration support)
export const FILTER_MAP: Record<TestCaseFilter, string[]> = {
  korean: [],
  math: [],
  science: [],
  music: [],
  english: [],
  'foreign-language': [],
  ipa: [],
  corpus: [],
}
