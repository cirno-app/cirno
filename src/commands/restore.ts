import { CAC } from 'cac'
import { join, resolve } from 'node:path'
import { rm } from 'node:fs/promises'
import { Cirno } from '../index.ts'
import { error, success, Tar } from '../utils.ts'

export default (cli: CAC) => cli
  .command('restore [backup]', 'Restore to a backup')
  .option('--cwd <path>', 'Specify the project folder')
  .action(async (id: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const app = cirno.get(id, 'restore')
    if (app.id === id) error('Cannot restore to a head instance.')
    const index = app.backups.findIndex(backup => backup.id === id)
    const backups = app.backups.splice(index)
    const tar = new Tar()
    const tar2 = new Tar()
    await tar.loadFile(join(cwd, 'apps', app.id + '.baka'), (header) => {
      const [part] = header.name.split('/', 1)
      if (app.backups.some(backup => backup.id === part)) return true
      return backups[0].id === part ? tar2 : false
    })
    for (const backup of app.backups.splice(index)) {
      delete cirno.apps[backup.id]
      delete cirno.state[app.id][backup.id]
    }
    await rm(join(cwd, 'apps', app.id), { recursive: true, force: true })
    await tar2.dumpDir(join(cwd, 'apps', app.id))
    if (app.backups.length) {
      await tar.dumpFile(join(cwd, 'apps', app.id + '.baka'))
    } else {
      await rm(join(cwd, 'apps', app.id + '.baka'))
    }
    await cirno.save()
    success(`App ${app.id} is successfully restored to backup ${id}.`)
  })
