#!/usr/bin/env node

import k from 'kleur'
import * as fs from 'node:fs/promises'
import { cac } from 'cac'
import { validate } from 'uuid'
import { createRequire } from 'node:module'
import { init, save } from './index.ts'

const require = createRequire(import.meta.url)
const { version } = require('../package.json')

function info(message: string) {
  console.log(k.bold(k.bgBlue(' INFO ') + ' ' + k.white(message)))
}

function error(message: string) {
  console.log(k.bold(k.bgRed(' Error ') + ' ' + k.white(message)))
}

function success(message: string) {
  console.log(k.bold(k.bgGreen(' SUCCESS ') + ' ' + k.white(message)))
}

const cli = cac('cirno').help().version(version)

cli
  .command('list', 'List all instances')
  .alias('ls')
  .option('--json', 'Output as JSON')
  .action(async (options) => {
    const cirno = await init(process.cwd())
    const instances = Object.values(cirno.instances)
    if (options.json) return console.log(JSON.stringify(instances))
    if (!instances.length) return info('No instances found.')
    info(`Found ${instances.length} instances:`)
    for (const instance of instances) {
      console.log(`${instance.id}\t${instance.name}`)
    }
  })

cli
  .command('import', 'Import an instance')
  .alias('i')

cli
  .command('export <id> <out>', 'Export an instance')

cli
  .command('clone <id>', 'Clone an instance')

cli
  .command('backup <id>', 'Backup an instance')

cli
  .command('restore <id> <parent>', 'Restore an instance')

cli
  .command('remove [id]', 'Remove an instance')
  .alias('rm')
  .action(async (id: string) => {
    const cirno = await init(process.cwd())
    if (!id) return error('Missing instance ID. See `cirno remove --help` for usage.')
    if (!validate(id)) return error('Invalid instance ID. See `cirno remove --help` for usage.')
    if (!cirno.instances[id]) return error(`Instance ${id} not found.`)
    await fs.rm(process.cwd() + '/instances/' + id, { recursive: true, force: true })
    delete cirno.instances[id]
    await save(process.cwd(), cirno)
    return success(`Instance ${id} is successfully removed.`)
  })

cli.parse()

if (!cli.matchedCommand && !cli.options.help) {
  cli.outputHelp()
}
