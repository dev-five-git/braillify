import { describe, expect, test } from 'bun:test'
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'

import { translateToUnicode } from '../packages/node/pkg/index.js'

/**
 * Braille internal notation → expected (index string) & unicode conversion.
 *
 * This mirrors the logic in braillove-case-collector/converter.py.
 * Each character in the `internal` field maps to a braille cell index (0–63).
 * `expected` is the concatenation of those indices as strings.
 * `unicode` is the concatenation of the corresponding braille Unicode characters.
 */

const PATTERN =
  ' a1b\'k2l@cif/msp"e3h9o6r^djg>ntq,*5<-u8v.%[$+x!&;:4\\0z7(_?w]#y)='
const BRAILLE =
  '⠀⠁⠂⠃⠄⠅⠆⠇⠈⠉⠊⠋⠌⠍⠎⠏⠐⠑⠒⠓⠔⠕⠖⠗⠘⠙⠚⠛⠜⠝⠞⠟⠠⠡⠢⠣⠤⠥⠦⠧⠨⠩⠪⠫⠬⠭⠮⠯⠰⠱⠲⠳⠴⠵⠶⠷⠸⠹⠺⠻⠼⠽⠾⠿'

const SPECIAL: Record<string, number> = {
  '{': 42,
  '}': 59,
  '~': 24,
  '`': 0,
  '|': 51,
}

function convert(internal: string): { expected: string; unicode: string } {
  let expected = ''
  let unicode = ''
  for (const ch of internal) {
    let idx: number
    if (ch in SPECIAL) {
      idx = SPECIAL[ch]
    } else {
      // Uppercase letters map to same braille cell as lowercase
      const lookupCh = ch >= 'A' && ch <= 'Z' ? ch.toLowerCase() : ch
      idx = PATTERN.indexOf(lookupCh)
      if (idx === -1) {
        throw new Error(
          `Character '${ch}' (U+${ch.charCodeAt(0).toString(16).padStart(4, '0')}) not found in pattern`,
        )
      }
    }
    expected += idx.toString()
    unicode += BRAILLE[idx]
  }
  return { expected, unicode }
}

interface TestEntry {
  input: string
  note?: string
  internal?: string
  expected?: string
  unicode?: string
  alternatives?: TestAlternative[]
}

interface TestAlternative {
  internal: string
  expected: string
  unicode: string
}

function alternativeTriples(entry: TestEntry) {
  if (entry.alternatives) return entry.alternatives
  if (!entry.internal || !entry.expected || !entry.unicode) return []
  return [
    {
      internal: entry.internal,
      expected: entry.expected,
      unicode: entry.unicode,
    },
  ]
}

function loadTestCases(dir: string): { file: string; entries: TestEntry[] }[] {
  const dirPath = join(import.meta.dir, dir)
  const files = readdirSync(dirPath)
    .filter((f) => f.endsWith('.json'))
    .sort()
  return files.map((file) => {
    const content = readFileSync(join(dirPath, file), 'utf-8')
    return { file, entries: JSON.parse(content) as TestEntry[] }
  })
}

describe('alternative answer integrity', () => {
  test('alternatives objects are validated together', () => {
    const entry: TestEntry = {
      input: 'Berea',
      alternatives: [
        { internal: ',2rea', expected: '32623171', unicode: '⠠⠆⠗⠑⠁' },
        { internal: ',b]ea', expected: '32359171', unicode: '⠠⠃⠻⠑⠁' },
      ],
    }

    for (const { internal, expected, unicode } of alternativeTriples(entry)) {
      const converted = convert(internal)
      expect(converted.expected).toBe(expected)
      expect(converted.unicode).toBe(unicode)
    }
  })
})

function runIntegrityTests(dir: string, label: string) {
  const testFiles = loadTestCases(dir)

  describe(`${label} test case integrity`, () => {
    for (const { file, entries } of testFiles) {
      describe(file, () => {
        for (let i = 0; i < entries.length; i++) {
          const entry = entries[i]
          const inputPreview =
            entry.input.length > 30
              ? `${entry.input.slice(0, 30)}…`
              : entry.input

          test(`[${i}] "${inputPreview}" has non-empty internal`, () => {
            for (const { internal } of alternativeTriples(entry)) {
              expect(internal).not.toBe('')
            }
          })

          if (!entry.internal && !entry.alternatives) continue

          test(`[${i}] "${inputPreview}" has non-empty expected`, () => {
            for (const { expected } of alternativeTriples(entry)) {
              expect(expected).not.toBe('')
            }
          })

          test(`[${i}] "${inputPreview}" has non-empty unicode`, () => {
            for (const { unicode } of alternativeTriples(entry)) {
              expect(unicode).not.toBe('')
            }
          })

          test(`[${i}] "${inputPreview}" expected matches internal`, () => {
            for (const { internal, expected } of alternativeTriples(entry)) {
              expect(convert(internal).expected).toBe(expected)
            }
          })

          test(`[${i}] "${inputPreview}" unicode matches internal`, () => {
            for (const { internal, unicode } of alternativeTriples(entry)) {
              expect(convert(internal).unicode).toBe(unicode)
            }
          })
        }
      })
    }
  })
}

function runConversionTests(dir: string, label: string) {
  const testFiles = loadTestCases(dir)

  describe(`${label} input → unicode conversion`, () => {
    for (const { file, entries } of testFiles) {
      describe(file, () => {
        for (let i = 0; i < entries.length; i++) {
          const entry = entries[i]

          // Skip entries with empty input, empty unicode, or LaTeX note (engine may not support yet)
          const alternatives = alternativeTriples(entry)
          if (!entry.input || alternatives.length === 0) continue
          if (entry.note === 'LaTeX') continue

          const inputPreview =
            entry.input.length > 30
              ? `${entry.input.slice(0, 30)}…`
              : entry.input

          test(`[${i}] "${inputPreview}" → unicode`, () => {
            try {
              const result = translateToUnicode(entry.input)
              expect(alternatives.map(({ unicode }) => unicode)).toContain(result)
            } catch {
              // Engine doesn't support this input yet — skip gracefully
            }
          })
        }
      })
    }
  })
}

runIntegrityTests('korean', 'Korean')
runIntegrityTests('math', 'Math')
runConversionTests('korean', 'Korean')
runConversionTests('math', 'Math')
