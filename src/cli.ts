#!/usr/bin/env node

import { cac } from 'cac'
import { createRequire } from 'node:module'
import registerInit from './commands/init.ts'
import registerCreate from './commands/create.ts'
import registerImport from './commands/import.ts'
import registerExport from './commands/export.ts'
import registerClone from './commands/clone.ts'
import registerRemove from './commands/remove.ts'
import registerBackup from './commands/backup.ts'
import registerRestore from './commands/restore.ts'
import registerList from './commands/list.ts'
import registerYarn from './commands/yarn.ts'
import registerGc from './commands/gc.ts'

const require = createRequire(import.meta.url)
const { version } = require('../package.json')

const cli = cac('cirno').help().version(version)

registerInit(cli)
registerCreate(cli)
registerImport(cli)
registerExport(cli)
registerClone(cli)
registerRestore(cli)
registerBackup(cli)
registerRemove(cli)
registerList(cli)
registerYarn(cli)
registerGc(cli)

cli.parse()

if (!cli.matchedCommand && !cli.options.help) {
  cli.outputHelp()
}
