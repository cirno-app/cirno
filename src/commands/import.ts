import { CAC } from 'cac'
import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { ZipFS } from '@yarnpkg/libzip'
import { parseSyml, stringifySyml } from '@yarnpkg/parsers'
import { Cirno, Package, YarnLock, YarnRc } from '../index.ts'
import { dumpFromZip, error, success } from '../utils.ts'
import * as fs from 'node:fs/promises'

function parseImport(src: string, cwd: string) {
  try {
    const url = new URL(src)
    if (url.protocol === 'file:') {
      return fileURLToPath(url)
    }
    return url
  } catch {
    return resolve(cwd, src)
  }
}

export default (cli: CAC) => cli
  .command('import [src] [name]', 'Import an instance')
  .option('--cwd <path>', 'Specify the project folder')
  .option('--id <id>', 'Specify the new instance ID')
  .action(async (src: string, name: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    if (!src) return error('Missing source path or url. See `cirno import --help` for usage.')
    const id = cirno.createId(options.id)
    const app = cirno.instances[id] = {
      id,
      name,
      backups: [],
    }
    const temp = cwd + '/temp/' + id
    const dest = cwd + '/apps/' + id
    await fs.mkdir(temp, { recursive: true })
    try {
      const parsed = parseImport(src, cwd)
      if (typeof parsed === 'string') {
        const stats = await fs.stat(parsed)
        if (stats.isDirectory()) {
          await fs.cp(parsed, temp, { recursive: true })
        } else {
          const buffer = await fs.readFile(parsed)
          await dumpFromZip(new ZipFS(buffer), temp)
        }
      } else {
        const response = await fetch(parsed)
        const buffer = Buffer.from(await response.arrayBuffer())
        await dumpFromZip(new ZipFS(buffer), temp)
      }

      const pkgMeta: Package = JSON.parse(await fs.readFile(temp + '/package.json', 'utf8'))
      app.name = name ?? pkgMeta.name

      // yarnPath
      const capture = /^yarn@(\d+\.\d+\.\d+)/.exec(pkgMeta.packageManager)
      if (!capture) throw new Error('Failed to detect yarn version.')
      const yarnRc: YarnRc = parseSyml(await fs.readFile(temp + '/.yarnrc.yml', 'utf8'))
      if (!yarnRc.yarnPath) throw new Error('Cannot find `yarnPath` in .yarnrc.yml.')
      const yarnPath = resolve(temp, yarnRc.yarnPath)
      await fs.rename(yarnPath, resolve(cwd, `.yarn/releases/yarn-${capture[1]}.cjs`))
      await fs.rm(resolve(temp, '.yarn/releases'), { recursive: true, force: true })
      delete yarnRc.yarnPath

      // enableGlobalCache
      const yarnLock = parseSyml(await fs.readFile(temp + '/yarn.lock', 'utf8')) as YarnLock
      const { version, cacheKey } = yarnLock.__metadata ?? {}
      if (version !== '8') throw new Error(`Unsupported yarn.lock version: ${version}.`)
      const files = await fs.readdir(resolve(temp, '.yarn/cache'))
      for (const name of files) {
        const capture = /^(.+)-([0-9a-f]{10})-([0-9a-f]+)\.zip$/.exec(name)
        if (!capture) continue
        await fs.rename(resolve(temp, '.yarn/cache', name), resolve(cwd, '.yarn/cache', `${capture[1]}-${capture[2]}-${cacheKey}.zip`))
      }
      await fs.rm(resolve(temp, '.yarn/cache'), { recursive: true, force: true })
      delete yarnRc.enableGlobalCache

      await fs.writeFile(temp + '/.yarnrc.yml', stringifySyml(yarnRc))
      await fs.rename(temp, dest)
      await cirno.save()
      await cirno.yarn(id, options['--'])
      success(`Successfully imported instance ${id}.`)
    } catch (e) {
      await fs.rm(temp, { recursive: true, force: true })
      error('Failed to import instance.', e)
    }
  })
