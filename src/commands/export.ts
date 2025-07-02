import { CAC } from 'cac'
import { resolve } from 'node:path'
import { Cirno, Package, YarnRc } from '../index.ts'
import { error, loadIntoZip, success } from '../utils.ts'
import { parseSyml, stringifySyml } from '@yarnpkg/parsers'
import { ZipFS } from '@yarnpkg/libzip'
import * as fs from 'node:fs/promises'

export default (cli: CAC) => cli
  .command('export [id] [dest]', 'Export an instance')
  .option('--cwd <path>', 'Specify the project folder')
  .option('--zip', 'Export as a zip file')
  .action(async (id: string, dest: string, options) => {
    // TODO: handle dependencies and modify yarnPath
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
