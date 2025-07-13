import { CAC } from 'cac'
import { join, resolve } from 'node:path'
import { rm } from 'node:fs/promises'
import { Cirno } from '../index.ts'
import { success, Tar } from '../utils.ts'
import { Pack } from 'tar-fs'

export default (cli: CAC) => cli
  .command('remove [id]', 'Remove an application or backup')
  .alias('rm')
  .option('--cwd <path>', 'Specify the root folder')
  .option('-r, --recursive', 'Remove backups recursively')
  .action(async (id: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const app = cirno.get(id, 'remove')
    const count = app.id === id
      ? Infinity
      : app.backups.findIndex(backup => backup.id === id)
    const tar = new Tar(join(cwd, 'baka', app.id + '.tar.br'))
    const oldLength = app.backups.length
    const backups = options.recursive
      ? app.backups.splice(0, count + 1)
      : app.backups.splice(count, 1)
    for (const backup of backups) {
      delete cirno.apps[backup.id]
      delete cirno.state[app.id][backup.id]
    }
    let pack: Pack | undefined
    if (app.id === id) {
      await rm(join(cwd, 'apps', id), { recursive: true, force: true })
      const restore = app.backups.pop()
      if (restore) {
        pack = tar.extract(join(cwd, 'apps', app.id), 1)
      } else {
        delete cirno.apps[id]
        delete cirno.state[id]
      }
    }
    if (app.backups.length || pack) {
      tar.load((header) => {
        const [part] = header.name.split('/', 1)
        if (app.backups.some(backup => backup.id === part)) return true
        return pack ?? false
      })
    }
    if (oldLength) {
      tar.dump(join(cwd, 'tmp', app.id + '.baka'), !!app.backups.length)
      await tar.finalize()
    }
    await cirno.save()
    await cirno.gc()
    success(`Instance ${id} is successfully removed.`)
  })
