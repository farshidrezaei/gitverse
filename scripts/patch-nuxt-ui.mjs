import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const rootDir = path.resolve(__dirname, '..')

function patchNuxtUIColors() {
  const searchPatterns = [
    path.join(rootDir, 'node_modules/@nuxt/ui/dist/runtime/plugins/colors.js'),
  ]

  // Also check pnpm virtual store if present
  const pnpmDir = path.join(rootDir, 'node_modules/.pnpm')
  if (fs.existsSync(pnpmDir)) {
    const entries = fs.readdirSync(pnpmDir)
    for (const entry of entries) {
      if (entry.startsWith('@nuxt+ui@')) {
        searchPatterns.push(path.join(pnpmDir, entry, 'node_modules/@nuxt/ui/dist/runtime/plugins/colors.js'))
      }
    }
  }

  for (const filePath of searchPatterns) {
    if (fs.existsSync(filePath)) {
      const content = fs.readFileSync(filePath, 'utf8')
      if (content.includes('head.hooks?.hook')) {
        const patched = content.replace('head.hooks?.hook', 'head?.hooks?.hook')
        fs.writeFileSync(filePath, patched, 'utf8')
      }
    }
  }
}

patchNuxtUIColors()
