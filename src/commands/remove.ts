import { CAC } from 'cac'
import { join, resolve } from 'node:path'
import { rename, rm } from 'node:fs/promises'
import { Backup, Cirno } from '../index.ts'
import { success, Tar } from '../utils.ts'
import { Pack } from 'tar-fs'

export default (cli: CAC) => cli
  .command('remove [id]', 'Remove an instance')
  .alias('rm')
  .option('--cwd <path>', 'Specify the project folder')
  .option('-r, --recursive', 'Remove backups recursively')
  .action(async (id: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const app = cirno.get(id, 'remove')
    const count = app.id === id
      ? Infinity
      : app.backups.findIndex(backup => backup.id === id)
    const tar = new Tar()
    const oldLength = app.backups.length
    const backups = options.recursive
      ? app.backups.splice(0, count + 1)
      : app.backups.splice(count, 1)
    for (const backup of backups) {
      delete cirno.apps[backup.id]
      delete cirno.state[app.id][backup.id]
    }
    let restore: Backup | undefined
    if (app.id === id) {
      await rm(join(cwd, 'apps', id), { recursive: true, force: true })
      restore = app.backups.pop()
      if (!restore) {
        delete cirno.apps[id]
        delete cirno.state[id]
      }
    }
    let pack: Pack | undefined
    if (restore) {
      pack = tar.createPack()
      tar.dumpDir(join(cwd, 'apps', app.id), 1, pack)
    }
    if (app.backups.length || restore) {
      tar.loadFile(join(cwd, 'apps', app.id + '.baka'), (header) => {
        const [part] = header.name.split('/', 1)
        if (app.backups.some(backup => backup.id === part)) return true
        return restore ? pack! : false
      })
    }
    const tmp = join(cwd, 'tmp', app.id + '.baka')
    if (oldLength && app.backups.length) {
      tar.dumpFile(tmp)
    }
    await tar.finalize()
    if (oldLength && app.backups.length) {
      await rename(tmp, join(cwd, 'apps', app.id + '.baka'))
    }
    if (oldLength && !app.backups.length) {
      await rm(join(cwd, 'apps', app.id + '.baka'))
    }
    await cirno.save()
    await cirno.gc()
    success(`Instance ${id} is successfully removed.`)
  })
