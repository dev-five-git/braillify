export type ApplicationField =
  'name' | 'product' | 'url' | 'purpose' | 'methods' | 'consent'
export type ApplicationErrors = Partial<Record<ApplicationField, string>>

export function validateApplication(data: {
  name: string
  product: string
  purpose: string
  methods: string[]
  consent: boolean
  urlInvalid: boolean
}): ApplicationErrors {
  const errors: ApplicationErrors = {}
  if (!data.name.trim()) errors.name = '이름을 입력해 주세요'
  if (!data.product.trim()) errors.product = '사용처 이름을 입력해 주세요'
  if (data.urlInvalid) errors.url = '올바른 주소를 입력해 주세요'
  if (!data.purpose.trim()) errors.purpose = '사용 용도를 입력해 주세요'
  if (!data.methods.length) errors.methods = '하나 이상 골라 주세요'
  if (!data.consent) errors.consent = '동의해야 신청할 수 있어요'
  return errors
}
