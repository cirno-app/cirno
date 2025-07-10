import { CAC } from 'cac'
import { join, resolve } from 'node:path'
import { readFile, rename, rm, writeFile } from 'node:fs/promises'
import { Cirno } from '../index.ts'
import { dumpFromZip, error, success } from '../utils.ts'
import { ZipFS } from '@yarnpkg/libzip'
import { PortablePath } from '@yarnpkg/fslib'

export default (cli: CAC) => cli
  .command('restore [backup]', 'Restore to a backup')
  .option('--cwd <path>', 'Specify the project folder')
  .action(async (id: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const app = cirno.get(id, 'restore')
    if (app.id === id) error('Cannot restore to a head instance.')
    const zip = new ZipFS(await readFile(join(cwd, 'apps', app.id + '.bak.zip')))
    const temp = join(cwd, 'tmp', id)
    await dumpFromZip(zip, temp, '/' + id + '/')
    const index = app.backups.findIndex(backup => backup.id === id)
    for (const backup of app.backups.splice(index)) {
      await zip.rmPromise('/' + backup.id as PortablePath, { recursive: true, force: true })
      delete cirno.apps[backup.id]
      delete cirno.state[app.id][backup.id]
    }
    await rm(join(cwd, 'apps', app.id), { recursive: true, force: true })
    await rename(temp, join(cwd, 'apps', app.id))
    if (app.backups.length) {
      await writeFile(join(cwd, 'apps', app.id + '.bak.zip'), zip.getBufferAndClose())
    } else {
      await rm(join(cwd, 'apps', app.id + '.bak.zip'))
    }
    await cirno.save()
    success(`App ${app.id} is successfully restored to backup ${id}.`)
  })
