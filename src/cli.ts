#!/usr/bin/env node

import * as fs from 'node:fs/promises'
import { PortablePath } from '@yarnpkg/fslib'
import { ZipFS } from '@yarnpkg/libzip'
import { cac } from 'cac'
import { createRequire } from 'node:module'
import { join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { Cirno } from './index.ts'
import { error, info, success } from './utils.ts'

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

async function dumpFromZip(zip: ZipFS, root: string, base = '/') {
  const dirents = await zip.readdirPromise(base as PortablePath, { withFileTypes: true })
  await Promise.all(dirents.map(async (dirent) => {
    if (dirent.isFile()) {
      const buffer = await zip.readFilePromise(base + dirent.name as PortablePath)
      await fs.writeFile(join(root, dirent.name), buffer)
    } else if (dirent.isDirectory()) {
      await fs.mkdir(join(root, dirent.name))
      await dumpFromZip(zip, join(root, dirent.name), base + dirent.name + '/')
    } else {
      throw new Error(`Unsupported file type`)
    }
  }))
}

async function loadIntoZip(zip: ZipFS, root: string, base = '/') {
  const dirents = await fs.readdir(root, { withFileTypes: true })
  await Promise.all(dirents.map(async (dirent) => {
    if (dirent.isFile()) {
      const buffer = await fs.readFile(join(root, dirent.name))
      await zip.writeFilePromise(base + dirent.name as PortablePath, buffer)
    } else if (dirent.isDirectory()) {
      await zip.mkdirPromise(base + dirent.name as PortablePath)
      await loadIntoZip(zip, join(root, dirent.name), base + dirent.name + '/')
    } else {
      throw new Error(`Unsupported file type`)
    }
  }))
}

cli
  .command('import [src] [name]', 'Import an instance')
  .option('--cwd <path>', 'Specify the project folder')
  .option('--id <id>', 'Specify the new instance ID')
  .action(async (src: string, name: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    if (!src) return error('Missing source path or url. See `cirno import --help` for usage.')
    const instance = cirno.create(name ?? 'unnamed', options.id)
    const dest = cwd + '/instances/' + instance.id
    await fs.mkdir(dest, { recursive: true })
    try {
      const parsed = parseImport(src, cwd)
      if (typeof parsed === 'string') {
        const stats = await fs.stat(parsed)
        if (stats.isDirectory()) {
          await fs.cp(parsed, dest, { recursive: true })
        } else {
          const buffer = await fs.readFile(parsed)
          await dumpFromZip(new ZipFS(buffer), dest)
        }
      } else {
        const response = await fetch(parsed)
        const buffer = Buffer.from(await response.arrayBuffer())
        await dumpFromZip(new ZipFS(buffer), dest)
      }
      await cirno.save()
      success(`Successfully imported instance ${instance.id}.`)
    } catch (e) {
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
      if (full.endsWith('.zip') || options.zip) {
        const zip = new ZipFS()
        await loadIntoZip(zip, cwd + '/instances/' + id)
        await fs.writeFile(full, zip.getBufferAndClose())
      } else {
        await fs.cp(cwd + '/instances/' + id, full, { recursive: true, force: true })
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
    const head = cirno.get(id, 'backup')
    if (!head) return
    const base = cirno.create(name ?? head.name, options.id)
    base.backup = { type: 'manual' }
    base.parent = head.parent
    head.parent = base.id
    await fs.cp(cwd + '/instances/' + id, cwd + '/instances/' + base.id, { recursive: true })
    success(`Successfully created a backup instance ${base.id}.`)
  })

cli
  .command('restore [head] [base]', 'Restore an instance')
  // .usage('Restore an instance into previous backup, deleting all intermediate instances.')
  .option('--cwd <path>', 'Specify the project folder')
  .action(async (headId: string, baseId: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const head = cirno.get(headId, 'restore')
    if (!head) return
    const base = cirno.get(baseId, 'restore')
    if (!base) return
    const instances = [...cirno.prepareRestore(head, base)]
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
    const instance = cirno.get(id, 'remove')
    if (!instance) return
    await fs.rm(cwd + '/instances/' + id, { recursive: true, force: true })
    delete cirno.instances[id]
    await cirno.save()
    success(`Instance ${id} is successfully removed.`)
  })

cli.parse()

if (!cli.matchedCommand && !cli.options.help) {
  cli.outputHelp()
}
