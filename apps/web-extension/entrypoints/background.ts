import { browser, defineBackground } from '#imports'

export default defineBackground(() => {
  console.info('Hello background!', { id: browser.runtime.id })
})
