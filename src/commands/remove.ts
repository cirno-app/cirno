import { CAC } from 'cac'
import { resolve } from 'node:path'
import { rename, rm } from 'node:fs/promises'
import { Cirno } from '../index.ts'
import { success } from '../utils.ts'

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
    const backups = options.recursive
      ? app.backups.splice(0, count + 1)
      : app.backups.splice(count, 1)
    for (const backup of backups) {
      await rm(cwd + '/instances/' + backup.id, { recursive: true, force: true })
      delete cirno.instances[backup.id]
    }
    if (app.id === id) {
      await rm(cwd + '/instances/' + id, { recursive: true, force: true })
      const backup = app.backups.pop()
      if (backup) {
        await rename(cwd + '/instances/' + backup.id, cwd + '/instances/' + id)
      } else {
        delete cirno.instances[id]
      }
    }
    await cirno.save()
    success(`Instance ${id} is successfully removed.`)
  })
