import { CAC } from 'cac'
import { resolve } from 'node:path'
import { rename, rm } from 'node:fs/promises'
import { Cirno } from '../index.ts'
import { error, success } from '../utils.ts'

export default (cli: CAC) => cli
  .command('restore [backup]', 'Restore to a backup')
  .option('--cwd <path>', 'Specify the project folder')
  .action(async (id: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const app = cirno.get(id, 'restore')
    if (app.id === id) error('Cannot restore to a head instance.')
    const index = app.backups.findIndex(backup => backup.id === id)
    for (const backup of app.backups.splice(index + 1)) {
      await rm(cwd + '/apps/' + backup.id, { recursive: true, force: true })
      delete cirno.instances[backup.id]
    }
    app.backups.pop()
    delete cirno.instances[id]
    await rm(cwd + '/apps/' + app.id, { recursive: true, force: true })
    await rename(cwd + '/apps/' + id, cwd + '/apps/' + app.id)
    await cirno.save()
    success(`App ${app.id} is successfully restored to backup ${id}.`)
  })
