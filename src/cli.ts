#!/usr/bin/env node

import * as fs from 'node:fs/promises'
import { ZipFS } from '@yarnpkg/libzip'
import { parseSyml, stringifySyml } from '@yarnpkg/parsers'
import { cac } from 'cac'
import { createRequire } from 'node:module'
import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { Cirno, Package, YarnRc } from './index.ts'
import { dumpFromZip, error, info, loadIntoZip, success } from './utils.ts'

const require = createRequire(import.meta.url)
const { version } = require('../package.json')

const cli = cac('cirno').help().version(version)

cli
  .command('init', 'Initialize a new project')
  .option('--cwd <path>', 'Specify the project folder')
  .option('-f, --force', 'Overwrite existing project')
  .action(async (options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd, true, options.force)
    await cirno.save()
    success(`Cirno project initialized at ${cwd}.`)
  })

cli
  .command('list', 'List all instances')
  .alias('ls')
  .option('--cwd <path>', 'Specify the project folder')
  .option('--json', 'Output as JSON')
  .action(async (options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const instances = Object.values(cirno.instances)
    if (options.json) return console.log(JSON.stringify(instances))
    if (!instances.length) return info('No instances found.')
    info(`Found ${instances.length} instances:`)
    for (const instance of instances) {
      console.log(`${instance.id}\t${instance.name}`)
    }
  })

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

cli
  .command('import [src] [name]', 'Import an instance')
  .option('--cwd <path>', 'Specify the project folder')
  .option('--id <id>', 'Specify the new instance ID')
  .action(async (src: string, name: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    if (!src) return error('Missing source path or url. See `cirno import --help` for usage.')
    const instance = cirno.create('', options.id)
    const temp = cwd + '/temp/' + instance.id
    const dest = cwd + '/instances/' + instance.id
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
      instance.name = name ?? pkgMeta.name

      // yarnPath
      const capture = /^yarn@(\d+\.\d+\.\d+)/.exec(pkgMeta.packageManager)
      if (!capture) throw new Error('Failed to detect yarn version.')
      const yarnRc: YarnRc = parseSyml(await fs.readFile(temp + '/.yarnrc.yml', 'utf8'))
      if (!yarnRc.yarnPath) throw new Error('Cannot find `yarnPath` in .yarnrc.yml.')
      const yarnPath = resolve(temp, yarnRc.yarnPath)
      await fs.rename(yarnPath, resolve(cwd, `.yarn/releases/yarn-${capture[1]}.cjs`))
      await fs.rm(resolve(temp, '.yarn/releases'), { recursive: true, force: true })
      delete yarnRc.yarnPath
      await fs.writeFile(temp + '/.yarnrc.yml', stringifySyml(yarnRc))

      await fs.rename(temp, dest)
      await cirno.save()
      success(`Successfully imported instance ${instance.id}.`)
    } catch (e) {
      await fs.rm(temp, { recursive: true, force: true })
      error('Failed to import instance.', e)
    }
  })

cli
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

cli
  .command('clone [id] [name]', 'Clone an instance')
  // .usage('Create a new instance with the same configuration as the base instance.')
  .option('--cwd <path>', 'Specify the project folder')
  .option('--id <id>', 'Specify the new instance ID')
  .action(async (id: string, name: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const base = cirno.get(id, 'clone')
    if (!base) return
    const head = cirno.create(name ?? base.name, options.id)
    head.backup = undefined
    await fs.cp(cwd + '/instances/' + id, cwd + '/instances/' + head.id, { recursive: true })
    await cirno.save()
    success(`Successfully created a cloned instance ${head.id}.`)
  })

cli
  .command('backup [id] [name]', 'Backup an instance')
  // .usage('Create a backup instance and link it to the base instance.')
  .option('--cwd <path>', 'Specify the project folder')
  .option('--id <id>', 'Specify the new instance ID')
  .action(async (id: string, name: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const head = cirno.head(id, 'backup')
    if (!head) return
    const base = cirno.create(name ?? head.name, options.id)
    base.backup = { type: 'manual' }
    base.parent = head.parent
    head.parent = base.id
    await fs.cp(cwd + '/instances/' + id, cwd + '/instances/' + base.id, { recursive: true })
    await cirno.save()
    success(`Successfully created a backup instance ${base.id}.`)
  })

cli
  .command('restore [head] [base]', 'Restore an instance')
  // .usage('Restore an instance into previous backup, deleting all intermediate instances.')
  .option('--cwd <path>', 'Specify the project folder')
  .action(async (headId: string, baseId: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const head = cirno.head(headId, 'restore')
    if (!head) return
    const base = cirno.get(baseId, 'restore')
    if (!base) return
    const instances = [...cirno.prepareRestore(head, baseId)]
    await Promise.all(instances.map(async (instance) => {
      await fs.rm(cwd + '/instances/' + instance.id, { recursive: true, force: true })
      delete cirno.instances[instance.id]
    }))
    base.backup = undefined
    await cirno.save()
    success(`Instance ${headId} is successfully restored to ${baseId}.`)
  })

cli
  .command('remove [id]', 'Remove an instance')
  .alias('rm')
  .option('--cwd <path>', 'Specify the project folder')
  .action(async (id: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    let instance = cirno.get(id, 'remove')
    if (!instance) return
    await fs.rm(cwd + '/instances/' + id, { recursive: true, force: true })
    delete cirno.instances[id]
    if (instance.backup) {
      for (const next of Object.values(cirno.instances)) {
        if (next.parent === id) {
          next.parent = instance.parent
          break
        }
      }
    } else {
      while (instance.parent) {
        instance = cirno.instances[instance.parent]
        await fs.rm(cwd + '/instances/' + instance.id, { recursive: true, force: true })
        delete cirno.instances[instance.id]
      }
    }
    await cirno.save()
    success(`Instance ${id} is successfully removed.`)
  })

cli.parse()

if (!cli.matchedCommand && !cli.options.help) {
  cli.outputHelp()
}
