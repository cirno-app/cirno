#!/usr/bin/env node

import * as fs from 'node:fs/promises'
import { cac } from 'cac'
import { createRequire } from 'node:module'
import { Cirno } from './index.ts'
import { resolve } from 'node:path'
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

cli
  .command('import', 'Import an instance')

cli
  .command('export [id] [out]', 'Export an instance')
  .action(async (id: string, out: string) => {
    // TODO: support .zip
    // TODO: handle dependencies and modify yarnPath
    const cirno = await Cirno.init(process.cwd())
    const instance = cirno.get(id)
    if (!instance) return
    if (!out) return error('Missing output path. See `cirno remove --help` for usage.')
    const outDir = resolve(cirno.cwd, out)
    await fs.cp(cirno.cwd + '/instances/' + id, outDir, { recursive: true, force: true })
    return success(`Instance ${id} is successfully exported to ${outDir}.`)
  })

cli
  .command('clone <id> [name]', 'Clone an instance')
  .action(async (id: string, name: string) => {
    const cirno = await Cirno.init(process.cwd())
    const old = cirno.get(id)
    if (!old) return
    const neo = cirno.create(name ?? old.name)
    await fs.cp(cirno.cwd + '/instances/' + id, cirno.cwd + '/instances/' + neo.id, { recursive: true })
    return success(`Successfully created a cloned instance ${neo.id}.`)
  })

cli
  .command('backup <id> [name]', 'Backup an instance')
  .action(async (id: string, name: string) => {
    const cirno = await Cirno.init(process.cwd())
    const old = cirno.get(id)
    if (!old) return
    const neo = cirno.create(name ?? old.name, { id })
    await fs.cp(cirno.cwd + '/instances/' + id, cirno.cwd + '/instances/' + neo.id, { recursive: true })
    return success(`Successfully created a backup instance ${neo.id}.`)
  })

cli
  .command('restore <id> <parent>', 'Restore an instance')

cli
  .command('remove [id]', 'Remove an instance')
  .alias('rm')
  .action(async (id: string) => {
    const cirno = await Cirno.init(process.cwd())
    const instance = cirno.get(id)
    if (!instance) return
    await fs.rm(cirno.cwd + '/instances/' + id, { recursive: true, force: true })
    delete cirno.data.instances[id]
    await cirno.save()
    return success(`Instance ${id} is successfully removed.`)
  })

cli.parse()

if (!cli.matchedCommand && !cli.options.help) {
  cli.outputHelp()
}
