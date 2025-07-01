#!/usr/bin/env node

import * as fs from 'node:fs/promises'
import AdmZip from 'adm-zip'
import { cac } from 'cac'
import { createRequire } from 'node:module'
import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { Cirno } from './index.ts'
import { error, info, success } from './utils.ts'

const require = createRequire(import.meta.url)
const { version } = require('../package.json')

const cli = cac('cirno').help().version(version)

cli
  .command('list', 'List all instances')
  .alias('ls')
  .option('--json', 'Output as JSON')
  .action(async (options) => {
    const cirno = await Cirno.init(process.cwd())
    const instances = Object.values(cirno.data.instances)
    if (options.json) return console.log(JSON.stringify(instances))
    if (!instances.length) return info('No instances found.')
    info(`Found ${instances.length} instances:`)
    for (const instance of instances) {
      console.log(`${instance.id}\t${instance.name}`)
    }
  })

function parseImport(src: string) {
  try {
    const url = new URL(src)
    if (url.protocol === 'file:') {
      return fileURLToPath(url)
    }
    return url
  } catch {
    return resolve(process.cwd(), src)
  }
}

async function extractZip(src: string | Buffer, dest: string) {
  const zip = new AdmZip(src)
  await new Promise<void>((resolve, reject) => {
    zip.extractAllToAsync(dest, true, undefined, (error) => {
      error ? reject(error) : resolve()
    })
  })
}

cli
  .command('import [src] [name]', 'Import an instance')
  // .option('-p, --password <password>', 'Password for encrypted zip file')
  .action(async (src: string, name: string, options) => {
    const cirno = await Cirno.init(process.cwd())
    if (!src) return error('Missing source path or url. See `cirno import --help` for usage.')
    const instance = cirno.create(name ?? 'unnamed')
    const dest = cirno.cwd + '/instances/' + instance.id
    try {
      const resolved = parseImport(src)
      if (typeof resolved === 'string') {
        const stats = await fs.stat(src)
        if (stats.isDirectory()) {
          await fs.cp(src, dest, { recursive: true })
        } else {
          await extractZip(src, dest)
        }
      } else {
        const response = await fetch(resolved)
        const buffer = await response.arrayBuffer()
        await extractZip(Buffer.from(buffer), dest)
      }
      await cirno.save()
      success(`Successfully imported instance ${instance.id}.`)
    } catch (e) {
      error('Failed to import instance.', e)
    }
  })

cli
  .command('export [id] [dest]', 'Export an instance')
  .option('--zip', 'Export as a zip file')
  // .option('-p, --password <password>', 'Password for encrypted zip file')
  .action(async (id: string, dest: string, options) => {
    // TODO: handle dependencies and modify yarnPath
    const cirno = await Cirno.init(process.cwd())
    const instance = cirno.get(id, 'export')
    if (!instance) return
    if (!dest) return error('Missing output path. See `cirno remove --help` for usage.')
    const full = resolve(cirno.cwd, dest)
    if (dest.endsWith('.zip') || options.zip) {
      const zip = new AdmZip()
      await zip.addLocalFolderPromise(cirno.cwd + '/instances/' + id, {})
      await zip.writeZipPromise(full, { overwrite: true })
    } else {
      await fs.cp(cirno.cwd + '/instances/' + id, full, { recursive: true, force: true })
    }
    success(`Successfully exported instance ${id} to ${full}.`)
  })

cli
  .command('clone [id] [name]', 'Clone an instance')
  // .usage('Create a new instance with the same configuration as the base instance.')
  .action(async (id: string, name: string) => {
    const cirno = await Cirno.init(process.cwd())
    const base = cirno.get(id, 'clone')
    if (!base) return
    const head = cirno.create(name ?? base.name)
    head.backup = undefined
    await fs.cp(cirno.cwd + '/instances/' + id, cirno.cwd + '/instances/' + head.id, { recursive: true })
    await cirno.save()
    success(`Successfully created a cloned instance ${head.id}.`)
  })

cli
  .command('backup [id] [name]', 'Backup an instance')
  // .usage('Create a backup instance and link it to the base instance.')
  .action(async (id: string, name: string) => {
    const cirno = await Cirno.init(process.cwd())
    const head = cirno.get(id, 'backup')
    if (!head) return
    const base = cirno.create(name ?? head.name)
    base.backup = { type: 'manual' }
    base.parent = head.parent
    head.parent = base.id
    await fs.cp(cirno.cwd + '/instances/' + id, cirno.cwd + '/instances/' + base.id, { recursive: true })
    success(`Successfully created a backup instance ${base.id}.`)
  })

cli
  .command('restore [head] [base]', 'Restore an instance')
  // .usage('Restore an instance into previous backup, deleting all intermediate instances.')
  .action(async (headId: string, baseId: string) => {
    const cirno = await Cirno.init(process.cwd())
    const head = cirno.get(headId, 'restore')
    if (!head) return
    const base = cirno.get(baseId, 'restore')
    if (!base) return
    const instances = [...cirno.prepareRestore(head, base)]
    await Promise.all(instances.map(async (instance) => {
      await fs.rm(cirno.cwd + '/instances/' + instance.id, { recursive: true, force: true })
      delete cirno.data.instances[instance.id]
    }))
    base.backup = undefined
    await cirno.save()
    success(`Instance ${headId} is successfully restored to ${baseId}.`)
  })

cli
  .command('remove [id]', 'Remove an instance')
  .alias('rm')
  .action(async (id: string) => {
    const cirno = await Cirno.init(process.cwd())
    const instance = cirno.get(id, 'remove')
    if (!instance) return
    await fs.rm(cirno.cwd + '/instances/' + id, { recursive: true, force: true })
    delete cirno.data.instances[id]
    await cirno.save()
    success(`Instance ${id} is successfully removed.`)
  })

cli.parse()

if (!cli.matchedCommand && !cli.options.help) {
  cli.outputHelp()
}
