import { CAC } from 'cac'
import { resolve } from 'node:path'
import { Cirno } from '../index.ts'
import { info } from '../utils.ts'

export default (cli: CAC) => cli
  .command('list', 'List all applications')
  .alias('ls')
  .alias('tree')
  .option('--cwd <path>', 'Specify the project folder')
  .option('--json', 'Output as JSON')
  .action(async (options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const apps = cirno.data.apps
    if (options.json) return console.log(JSON.stringify(apps))
    if (!apps.length) return info('No applications found.')
    info(`Found ${apps.length} applications:`)
    apps.forEach((app, index) => {
      const prefix = index === apps.length - 1 ? '└' : '├'
      console.log(`${prefix}── ${app.id}\t${app.name}`)
      app.backups.forEach((backup, index) => {
        const prefix = index === app.backups.length - 1 ? '└' : '├'
        console.log(`    ${prefix}── ${backup.id}`)
      })
    })
  })
