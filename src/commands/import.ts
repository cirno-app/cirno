import { CAC } from 'cac'
import { join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { ZipFS } from '@yarnpkg/libzip'
import { stringifySyml } from '@yarnpkg/parsers'
import { Cirno, loadMeta } from '../index.ts'
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
  .command('import [src]', 'Import an instance')
  .option('--cwd <path>', 'Specify the root folder')
  .option('--id <id>', 'Specify the new instance ID')
  .option('--name <name>', 'Specify the new application name')
  .action(async (src: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    if (!src) return error('Missing source path or url. See `cirno import --help` for usage.')

    const temp = cwd + '/tmp/' + Math.random().toString(36).slice(2, 10).padEnd(8, '0')
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

      const { pkg, yarnLock, yarnRc } = await loadMeta(temp)
      const name = options.name || pkg.name

      // yarnPath
      if (!pkg.packageManager) error('Missing `packageManager` in package.json.')
      const capture = /^yarn@(\d+\.\d+\.\d+)/.exec(pkg.packageManager)
      if (!capture) error(`Unsupported package manager: ${pkg.packageManager}.`)
      if (yarnRc.yarnPath) {
        const yarnPath = resolve(temp, yarnRc.yarnPath)
        await fs.rename(yarnPath, join(cwd, `home/.yarn/releases/yarn-${capture[1]}.cjs`))
        await fs.rm(join(temp, '.yarn/releases'), { recursive: true, force: true })
        delete yarnRc.yarnPath
      } else {
        await cirno.downloadYarn(capture[1], yarnRc.npmRegistryServer)
      }

      // cacheFolder, enableGlobalCache
      const { version, cacheKey } = yarnLock.__metadata ?? {}
      if (version !== '8') throw new Error(`Unsupported yarn.lock version: ${version}.`)
      let cacheFolder: string | undefined
      if (yarnRc.enableGlobalCache !== 'true') {
        cacheFolder = resolve(temp, yarnRc.cacheFolder ?? '.yarn/cache')
        if (!cacheFolder.startsWith(temp)) cacheFolder = undefined
      }
      if (cacheFolder) {
        const files = await fs.readdir(cacheFolder)
        for (const name of files) {
          const capture = /^(.+)-([0-9a-f]{10})-([0-9a-f]+)\.zip$/.exec(name)
          if (!capture) continue
          await fs.rename(join(cacheFolder, name), join(cwd, 'home/.yarn/cache', `${capture[1]}-${capture[2]}-${cacheKey}.zip`))
        }
        await fs.rm(cacheFolder, { recursive: true, force: true })
      }
      delete yarnRc.cacheFolder
      delete yarnRc.enableGlobalCache

      await fs.writeFile(temp + '/.yarnrc.yml', stringifySyml(yarnRc))
      const code = await cirno.yarn(temp, options['--'])
      if (code !== 0) error(`Failed to install dependencies. Exit code: ${code}`)

      const id = cirno.createId(options.id)
      cirno.apps[id] = {
        id,
        name,
        created: new Date().toISOString(),
        backups: [],
      }
      cirno.state[id] = {}
      await fs.rename(temp, join(cwd, 'apps', id))
      await cirno.save()
      success(`Successfully imported instance ${id}.`)
    } catch (e) {
      await fs.rm(temp, { recursive: true, force: true })
      error('Failed to import instance.', e)
    }
  })
