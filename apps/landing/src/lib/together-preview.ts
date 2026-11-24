export function previewDescription(
  name: string,
  kind: string,
  braille: string,
) {
  return `미리 보기: ${name || '이름을 입력하면 여기에 표시돼요'}, ${kind}, 점자 ${braille}`
}
