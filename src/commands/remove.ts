import { CAC } from 'cac'
import { join, resolve } from 'node:path'
import { readFile, rm, writeFile } from 'node:fs/promises'
import { Cirno } from '../index.ts'
import { dumpFromZip, success } from '../utils.ts'
import { ZipFS } from '@yarnpkg/libzip'
import { PortablePath } from '@yarnpkg/fslib'

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
    const zip = app.backups.length
      ? new ZipFS(await readFile(join(cwd, 'apps', app.id + '.bak.zip')))
      : undefined
    const backups = options.recursive
      ? app.backups.splice(0, count + 1)
      : app.backups.splice(count, 1)
    for (const backup of backups) {
      await zip!.rmPromise('/' + backup.id as PortablePath, { recursive: true, force: true })
      delete cirno.apps[backup.id]
      delete cirno.state[app.id][backup.id]
    }
    if (app.id === id) {
      await rm(join(cwd, 'apps', id), { recursive: true, force: true })
      const backup = app.backups.pop()
      if (backup) {
        await dumpFromZip(zip!, join(cwd, 'apps', id), '/' + backup.id + '/')
        await zip!.rmPromise('/' + backup.id as PortablePath, { recursive: true, force: true })
      } else {
        delete cirno.apps[id]
        delete cirno.state[id]
      }
    }
    if (app.backups.length) {
      await writeFile(join(cwd, 'apps', app.id + '.bak.zip'), zip!.getBufferAndClose())
    } else if (zip) {
      await rm(join(cwd, 'apps', app.id + '.bak.zip'))
    }
    await cirno.save()
    await cirno.gc()
    success(`Instance ${id} is successfully removed.`)
  })
