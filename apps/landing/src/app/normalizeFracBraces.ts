/**
 * MathLive는 한 글자 인자를 중괄호 없이 직렬화한다 (예: \frac34, \frac{3}4).
 * braillify의 LaTeX 파서는 \frac{분자}{분모} 형태만 분수로 인식하므로
 * \frac의 두 인자를 항상 중괄호로 감싼 정규형으로 변환한다.
 */
export function normalizeFracBraces(latex: string): string {
  let out = ''
  let i = 0
  while (i < latex.length) {
    if (latex.startsWith('\\frac', i) && !/[a-zA-Z]/.test(latex[i + 5] ?? '')) {
      const [num, j] = readArg(latex, i + 5)
      const [den, k] = readArg(latex, j)
      out += `\\frac${num}${den}`
      i = k
      continue
    }
    out += latex[i]
    i += 1
  }
  return out
}

/** i 위치부터 LaTeX 인자 하나를 읽어 중괄호로 감싼 형태와 다음 인덱스를 돌려준다. */
function readArg(latex: string, i: number): [string, number] {
  while (latex[i] === ' ') i += 1
  if (latex[i] === '{') {
    let depth = 0
    let j = i
    do {
      if (latex[j] === '{') depth += 1
      else if (latex[j] === '}') depth -= 1
      j += 1
    } while (j < latex.length && depth > 0)
    // depth === 0 이면 짝이 맞는 '}' 를 j-1 에서 소비한 것이고,
    // depth > 0 이면 닫는 중괄호 없이 끝난 것이라 내용을 j 까지 살린다.
    const end = depth === 0 ? j - 1 : j
    return [`{${normalizeFracBraces(latex.slice(i + 1, end))}}`, j]
  }
  if (latex[i] === '\\') {
    let j = i + 1
    while (j < latex.length && /[a-zA-Z]/.test(latex[j] ?? '')) j += 1
    // 제어기호(\, \! 등)는 백슬래시 뒤에 글자가 없으므로 기호 한 글자를 포함시킨다.
    if (j === i + 1 && j < latex.length) j += 1
    return [`{${latex.slice(i, j)}}`, j]
  }
  return [`{${latex[i] ?? ''}}`, i + 1]
}
