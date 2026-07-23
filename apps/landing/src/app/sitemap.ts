import type { MetadataRoute } from 'next'

// Required for `output: 'export'` in Next.js — without this, Next refuses
// to pre-render the route handler at build time.
export const dynamic = 'force-static'

const SITE_URL = 'https://braillify.kr'

/**
 * Static sitemap. Compatible with `output: 'export'`.
 * Generated at build time as /sitemap.xml.
 *
 * When a new route is added under `app/`, add a matching entry here so
 * crawlers can discover it. Keep `priority` and `changeFrequency`
 * meaningful (homepage > docs > leaf pages).
 */
export default function sitemap(): MetadataRoute.Sitemap {
  const now = new Date()

  return [
    {
      url: `${SITE_URL}/`,
      lastModified: now,
      changeFrequency: 'weekly',
      priority: 1.0,
    },
    {
      url: `${SITE_URL}/docs/overview`,
      lastModified: now,
      changeFrequency: 'weekly',
      priority: 0.9,
    },
    {
      url: `${SITE_URL}/docs/installation`,
      lastModified: now,
      changeFrequency: 'weekly',
      priority: 0.8,
    },
    {
      url: `${SITE_URL}/docs/api`,
      lastModified: now,
      changeFrequency: 'weekly',
      priority: 0.8,
    },
    {
      url: `${SITE_URL}/docs/contributing`,
      lastModified: now,
      changeFrequency: 'monthly',
      priority: 0.6,
    },
    {
      url: `${SITE_URL}/test-case`,
      lastModified: now,
      changeFrequency: 'daily',
      priority: 0.8,
    },
    {
      url: `${SITE_URL}/team`,
      lastModified: now,
      changeFrequency: 'monthly',
      priority: 0.5,
    },
  ]
}
