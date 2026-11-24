import { lstat, mkdtemp, mkdir, readFile, realpath, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { spawnSync } from 'node:child_process'
import { fileURLToPath, pathToFileURL } from 'node:url'

import type { Partner } from './together-data'

import { brailleSvg, escapeHtml, translateName } from './together-image-utils'
export { brailleSvg, escapeHtml, pruneTogetherImages, translateName } from './together-image-utils'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const landing = join(root, 'apps/landing')

export async function generateTogetherImages(partners: Partner[], publicDir: string) {
  if (!partners.length) return
  const safeRoot = await realpath(publicDir)
  const chrome = process.env.CHROME_PATH || Bun.which('google-chrome') || Bun.which('chromium') || (process.platform === 'win32' ? 'C:/Program Files/Google/Chrome/Application/chrome.exe' : process.platform === 'darwin' ? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' : '')
  if (!chrome) throw new Error('Chrome is required to generate together images')
  const executable = join(root, 'target', 'release', process.platform === 'win32' ? 'braillify.exe' : 'braillify')
  const bold = (await readFile(join(landing, 'assets/together/SpoqaHanSansNeo-Bold.woff2'))).toString('base64')
  const medium = (await readFile(join(landing, 'assets/together/SpoqaHanSansNeo-Medium.woff2'))).toString('base64')
  const source = await readFile(join(landing, 'src/components/icons/IconBraillify.tsx'), 'utf8')
  const icon = source.match(/<svg[\s\S]*?<\/svg>/)?.[0].replace('{...props}', '')
  if (!icon) throw new Error('Braillify logo unavailable')
  const logo = `data:image/svg+xml;base64,${Buffer.from(icon).toString('base64')}`
  const fonts = `@font-face{font-family:Spoqa;src:url(data:font/woff2;base64,${bold});font-weight:700}@font-face{font-family:Spoqa;src:url(data:font/woff2;base64,${medium});font-weight:500}`
  const work = await mkdtemp(join(tmpdir(), 'braillify-cert-'))
  try {
    const render = async (body: string, width: number, height: number, output: string, scale: number, checkLogo = false) => {
      const page = join(work, 'page.html')
      await writeFile(page, `<!doctype html><html><head><meta charset="utf-8"><style>${fonts}*{box-sizing:border-box}html,body{margin:0;width:${width}px;height:${height}px;font-family:Spoqa,sans-serif}body{overflow:hidden}</style></head><body>${body}</body></html>`)
      await mkdir(dirname(output), { recursive: true })
      const parent = await realpath(dirname(output))
      if (!parent.startsWith(`${safeRoot}${process.platform === 'win32' ? '\\' : '/'}`)) throw new Error('Image output is outside public directory')
      try { if ((await lstat(output)).isSymbolicLink()) throw new Error('Image output cannot be a symlink') } catch (error) { if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error }
      const args = async () => ['--headless=new', '--disable-gpu', '--hide-scrollbars', '--no-first-run', `--user-data-dir=${await mkdtemp(join(work, 'profile-'))}`, `--window-size=${width},${height}`, `--force-device-scale-factor=${scale}`, '--virtual-time-budget=3000']
      if (checkLogo) {
        const check = spawnSync(chrome, [...await args(), '--dump-dom', pathToFileURL(page).href], { encoding: 'utf8', timeout: 60000, maxBuffer: 8 * 1024 * 1024 })
        if (check.error || check.status !== 0) throw new Error(`Chrome logo check failed: ${check.error || check.stderr || `exit ${check.status}`}`)
        if (!check.stdout.includes('data-logo-status="ok"')) throw new Error('Partner logo cannot be decoded or is too large')
      }
      const result = spawnSync(chrome, [...await args(), `--screenshot=${output}`, pathToFileURL(page).href], { encoding: 'utf8', timeout: 60000 })
      if (result.error || result.status !== 0) throw new Error(`Chrome image generation failed: ${result.error || result.stderr || `exit ${result.status}`}`)
      const png = await readFile(output)
      if (png.length < 24 || png.readUInt32BE(16) !== width * scale || png.readUInt32BE(20) !== height * scale) throw new Error('Unexpected generated image dimensions')
    }
    for (const partner of partners) {
      const braille = brailleSvg(translateName(executable, partner.name))
      const image = await readFile(join(publicDir, partner.logo.slice(1)))
      const mime = partner.logo.endsWith('.png') ? 'image/png' : 'image/jpeg'
      const userLogo = `data:${mime};base64,${image.toString('base64')}`
      const name = escapeHtml(partner.name), kind = escapeHtml(partner.kind)
      const [year, month] = partner.approvedAt.split('-')
      const date = `${year}년 ${Number(month)}월`
      const subject = partner.kind === '기업' || partner.kind === '기관' ? '은' : '는'
      const cert = `<div style="background:#EFEEEB;width:1200px;height:630px;padding:40px"><div style="background:#fff;border:3px solid #2B2B2B;border-radius:28px;height:100%;padding:40px 48px;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:32px"><div style="display:flex;align-items:center;gap:20px;width:100%;padding-bottom:8px"><div style="flex:1;text-align:right"><img src="${logo}" width="197" height="85"></div><span style="color:#787878;font-size:24px">×</span><div style="flex:1"><img id="partner-logo" src="${userLogo}" style="max-width:240px;max-height:96px;object-fit:contain"></div></div><div style="text-align:center;display:flex;flex-direction:column;gap:12px;align-items:center;width:100%"><div style="font-size:52px;font-weight:700;line-height:64px;letter-spacing:-.06em;word-break:keep-all;text-wrap:balance">${name}</div>${braille}</div><div style="width:64px;height:2px;background:#2B2B2B"></div><div style="text-align:center;font-weight:500;display:flex;flex-direction:column;gap:12px"><div style="font-size:28px">위 ${kind}${subject} Braillify와 함께하는 ${kind}입니다.</div><div style="font-size:20px;color:#666">braillify.kr/together · ${date} 승인</div></div></div></div><script>const img=document.getElementById('partner-logo');function trim(){if(img.dataset.trimmed)return;img.dataset.trimmed='1';const c=document.createElement('canvas'),w=img.naturalWidth,h=img.naturalHeight;if(!w||!h||w*h>16777216)return;c.width=w;c.height=h;const ctx=c.getContext('2d');ctx.drawImage(img,0,0);const p=ctx.getImageData(0,0,w,h).data;let l=w,t=h,r=-1,b=-1;for(let y=0;y<h;y++)for(let x=0;x<w;x++)if(p[(y*w+x)*4+3]){l=Math.min(l,x);r=Math.max(r,x);t=Math.min(t,y);b=Math.max(b,y)}if(r<l)return;const out=document.createElement('canvas');out.width=r-l+1;out.height=b-t+1;out.getContext('2d').drawImage(c,l,t,out.width,out.height,0,0,out.width,out.height);img.src=out.toDataURL()}</script><script>img.addEventListener('load',trim);if(img.complete)trim()</script>`
      const badge = (dark: boolean) => `<div style="width:300px;height:80px;display:flex;align-items:center;gap:16px;padding:0 20px;border-radius:12px;background:${dark ? '#2B2B2B' : '#fff'};${dark ? '' : 'border:1px solid #E0E0E0;'}"><img src="${logo}" width="${dark ? 88 : 83}" height="36" style="${dark ? 'filter:invert(1)' : ''}"><div style="width:1px;height:32px;background:${dark ? '#5E5E5E' : '#E0E0E0'}"></div><div style="display:flex;flex-direction:column;gap:2px;letter-spacing:-.06em"><div style="font-size:16px;line-height:22px;font-weight:700;color:${dark ? '#fff' : '#1A1A1A'}">함께하는 ${kind}</div><div style="font-size:12px;line-height:17px;font-weight:500;color:${dark ? '#BDBDBD' : '#666'}">${date} · braillify.kr</div></div></div>`
      for (const scale of [1, 2]) {
        const suffix = scale === 2 ? '@2x' : ''
        const checkLogo = `<script>window.addEventListener('load',()=>setTimeout(()=>{const i=document.getElementById('partner-logo'),w=i.naturalWidth,h=i.naturalHeight;if(!w||!h||w*h>16777216){document.body.dataset.logoStatus='invalid';return}const c=document.createElement('canvas');c.width=w;c.height=h;const x=c.getContext('2d');x.drawImage(i,0,0);const p=x.getImageData(0,0,w,h).data;let found=false;for(let n=3;n<p.length;n+=4)if(p[n]){found=true;break}document.body.dataset.logoStatus=found?'ok':'invalid'},100))</script>`
        await render(cert + checkLogo, 1200, 630, join(publicDir, 'together/cert', `org-${partner.issue}${suffix}.png`), scale, true)
        for (const color of ['light', 'dark']) await render(badge(color === 'dark'), 300, 80, join(publicDir, 'together/badge', `org-${partner.issue}-${color}${suffix}.png`), scale)
      }
    }
  } finally { await rm(work, { recursive: true, force: true }) }
}
