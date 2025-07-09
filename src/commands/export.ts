import { CAC } from 'cac'
import { join, resolve } from 'node:path'
import { Cirno, getCacheFiles, loadMeta } from '../index.ts'
import { error, loadIntoZip, success } from '../utils.ts'
import { stringifySyml } from '@yarnpkg/parsers'
import { ZipFS } from '@yarnpkg/libzip'
import * as fs from 'node:fs/promises'

function formatSize(size: number) {
  const units = ['B', 'KB', 'MB', 'GB']
  for (const idx in units) {
    if (idx && size > 1024) {
      size /= 1024
    } else {
      return `${+size.toFixed(1)} ${units[idx]}`
    }
  }
  return `${+size.toFixed(1)} ${units[units.length - 1]}`
}

export default (cli: CAC) => cli
  .command('export [id] [dest]', 'Export an instance')
  .option('--cwd <path>', 'Specify the project folder')
  .option('--zip', 'Export as a zip file')
  .action(async (id: string, dest: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    cirno.get(id, 'export')
    if (!dest) return error('Missing output path. See `cirno remove --help` for usage.')
    try {
      const full = join(cwd, dest)
      const temp = cwd + '/tmp/' + id
      await fs.cp(cwd + '/apps/' + id, temp, { recursive: true, force: true })
      const { pkg, yarnLock, yarnRc } = await loadMeta(temp)

      // yarnPath
      const capture = /^yarn@(\d+\.\d+\.\d+)/.exec(pkg.packageManager)
      if (!capture) throw new Error('Failed to detect yarn version.')
      yarnRc.yarnPath = `.yarn/releases/yarn-${capture[1]}.cjs`
      await fs.mkdir(join(temp, '.yarn/releases'), { recursive: true })
      await fs.cp(join(cwd, `home/.yarn/releases/yarn-${capture[1]}.cjs`), join(temp, yarnRc.yarnPath))

      // enableGlobalCache
      const { version, cacheKey } = yarnLock.__metadata ?? {}
      if (version !== '8') throw new Error(`Unsupported yarn.lock version: ${version}.`)
      await fs.mkdir(join(temp, '.yarn/cache'), { recursive: true })
      const cache = (await cirno.loadCache())[cacheKey] ?? {}
      for (const prefix of getCacheFiles(yarnLock)) {
        const name = cache[prefix]
        if (!name) throw new Error(`Cache not found: ${prefix}`)
        await fs.cp(join(cwd, 'home/.yarn/cache', name), join(temp, '.yarn/cache', name))
      }
      yarnRc.enableGlobalCache = false

      await fs.writeFile(temp + '/.yarnrc.yml', stringifySyml(yarnRc))

      let size = ''
      if (full.endsWith('.zip') || options.zip) {
        const zip = new ZipFS()
        await loadIntoZip(zip, temp)
        await fs.rm(temp, { recursive: true, force: true })
        const buffer = zip.getBufferAndClose()
        await fs.writeFile(full, buffer)
        size = ` (${formatSize(buffer.byteLength)})`
      } else {
        await fs.rename(temp, full)
      }
      success(`Successfully exported instance ${id} to ${full}${size}.`)
    } catch (e) {
      error('Failed to export instance.', e)
    }
  })
