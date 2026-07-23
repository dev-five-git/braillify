import type { MetadataRoute } from 'next'

// Required for `output: 'export'` in Next.js — without this, Next refuses
// to pre-render the route handler at build time.
export const dynamic = 'force-static'

const SITE_URL = 'https://braillify.kr'

/**
 * Static robots.txt. Compatible with `output: 'export'`.
 * Generated at build time as /robots.txt.
 *
 * Strategy:
 *   - Allow everything for general crawlers (small marketing site, no admin).
 *   - Disallow Next.js internals (/_next/) just to keep crawl budget tight.
 *   - Explicitly opt-in major search engines (Google, Bing, Naver Yeti, Daum,
 *     DuckDuckGo, Yandex) and AI training / answer engines so the project
 *     and its open-source documentation get indexed and cited.
 *   - Braillify is Apache-2.0 licensed; AI training opt-in matches the
 *     project's "open knowledge" stance. Flip individual entries to
 *     `disallow: '/'` if the org's stance changes.
 *   - Point at the sitemap and the LLM-friendly entry files for discovery.
 */
export default function robots(): MetadataRoute.Robots {
  const allowAll = (userAgent: string) => ({
    userAgent,
    allow: '/',
  })

  return {
    rules: [
      // ── Default policy ────────────────────────────────────────────────
      {
        userAgent: '*',
        allow: '/',
        disallow: ['/_next/', '/api/'],
      },

      // ── Search engines ────────────────────────────────────────────────
      allowAll('Googlebot'),
      allowAll('Googlebot-Image'),
      allowAll('Googlebot-News'),
      allowAll('GoogleOther'),
      allowAll('bingbot'),
      allowAll('Yeti'), // Naver — critical for Korean SEO
      allowAll('Daum'), // Daum / Kakao
      allowAll('DuckDuckBot'),
      allowAll('YandexBot'),
      allowAll('Applebot'),

      // ── AI training & answer engines (explicit opt-in) ────────────────
      // OpenAI
      allowAll('GPTBot'),
      allowAll('OAI-SearchBot'),
      allowAll('ChatGPT-User'),

      // Anthropic
      allowAll('ClaudeBot'),
      allowAll('Claude-Web'),
      allowAll('anthropic-ai'),

      // Google (Gemini / Bard / Vertex)
      allowAll('Google-Extended'),

      // Perplexity
      allowAll('PerplexityBot'),
      allowAll('Perplexity-User'),

      // Apple Intelligence
      allowAll('Applebot-Extended'),

      // Meta AI
      allowAll('Meta-ExternalAgent'),
      allowAll('FacebookBot'),

      // Amazon (Alexa / Rufus)
      allowAll('Amazonbot'),

      // ByteDance (Doubao / TikTok search)
      allowAll('Bytespider'),

      // Common Crawl (training data source for many LLMs)
      allowAll('CCBot'),

      // Cohere
      allowAll('cohere-ai'),
      allowAll('cohere-training-data-crawler'),

      // You.com
      allowAll('YouBot'),

      // Diffbot
      allowAll('Diffbot'),

      // Mistral
      allowAll('MistralAI-User'),

      // Naver AI (CLOVA)
      allowAll('NaverBot'),
    ],
    sitemap: `${SITE_URL}/sitemap.xml`,
    host: SITE_URL,
  }
}
