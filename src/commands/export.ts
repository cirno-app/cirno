import { CAC } from 'cac'
import { resolve } from 'node:path'
import { Cirno, Package, YarnLock, YarnRc } from '../index.ts'
import { error, loadIntoZip, success } from '../utils.ts'
import { parseSyml, stringifySyml } from '@yarnpkg/parsers'
import { ZipFS } from '@yarnpkg/libzip'
import * as fs from 'node:fs/promises'

export default (cli: CAC) => cli
  .command('export [id] [dest]', 'Export an instance')
  .option('--cwd <path>', 'Specify the project folder')
  .option('--zip', 'Export as a zip file')
  .action(async (id: string, dest: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const instance = cirno.get(id, 'export')
    if (!instance) return
    if (!dest) return error('Missing output path. See `cirno remove --help` for usage.')
    try {
      const full = resolve(cwd, dest)
      const temp = cwd + '/temp/' + id
      await fs.cp(cwd + '/instances/' + id, temp, { recursive: true, force: true })

      // yarnPath
      const pkgMeta: Package = JSON.parse(await fs.readFile(temp + '/package.json', 'utf8'))
      const capture = /^yarn@(\d+\.\d+\.\d+)/.exec(pkgMeta.packageManager)
      if (!capture) throw new Error('Failed to detect yarn version.')
      const yarnRc: YarnRc = parseSyml(await fs.readFile(temp + '/.yarnrc.yml', 'utf8'))
      yarnRc.yarnPath = `.yarn/releases/yarn-${capture[1]}.cjs`
      await fs.mkdir(resolve(temp, '.yarn/releases'), { recursive: true })
      await fs.cp(resolve(cwd, `.yarn/releases/yarn-${capture[1]}.cjs`), resolve(temp, yarnRc.yarnPath))

      // enableGlobalCache
      const yarnLock = parseSyml(await fs.readFile(temp + '/yarn.lock', 'utf8')) as YarnLock
      await fs.mkdir(resolve(temp, '.yarn/cache'), { recursive: true })
      const files = await fs.readdir(resolve(cwd, '.yarn/cache'))
      const cacheMap: Record<string, string> = Object.create(null)
      for (const name of files) {
        const capture = /^(.+)-[0-9a-f]{10}-[0-9a-f]{10}\.zip$/.exec(name)
        if (!capture) continue
        cacheMap[capture[1]] = name
      }
      const { version } = yarnLock.__metadata ?? {}
      if (version !== '8') throw new Error(`Unsupported yarn.lock version: ${version}.`)
      for (const [key, value] of Object.entries(yarnLock)) {
        if (key === '__metadata') continue
        const capture = /^(@[^@/]+\/[^@]+|[^@/]+)@([^:]+):(.+)$/.exec(value.resolution)
        if (!capture) throw new Error(`Failed to parse resolution: ${value.resolution}`)
        if (capture[2] === 'workspace') continue
        if (capture[2] !== 'npm') throw new Error(`Unsupported resolution protocol: ${capture[2]}`)
        const name = cacheMap[capture[1].replace('/', '-') + '-' + capture[2] + '-' + capture[3].replace(':', '-')]
        if (!name) throw new Error(`Cache not found: ${value.resolution}`)
        await fs.rename(resolve(cwd, '.yarn/cache', name), resolve(temp, '.yarn/cache', name))
      }
      yarnRc.enableGlobalCache = false

      await fs.writeFile(temp + '/.yarnrc.yml', stringifySyml(yarnRc))

      if (full.endsWith('.zip') || options.zip) {
        const zip = new ZipFS()
        await loadIntoZip(zip, temp)
        await fs.rm(temp, { recursive: true, force: true })
        await fs.writeFile(full, zip.getBufferAndClose())
      } else {
        await fs.rename(temp, full)
      }
      success(`Successfully exported instance ${id} to ${full}.`)
    } catch (e) {
      error('Failed to export instance.', e)
    }
  })
