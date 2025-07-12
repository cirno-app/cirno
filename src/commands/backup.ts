import { CAC } from 'cac'
import { join, resolve } from 'node:path'
import { Cirno, loadMeta } from '../index.ts'
import { error, success, Tar } from '../utils.ts'
import { rename } from 'node:fs/promises'

export default (cli: CAC) => cli
  .command('backup [id]', 'Backup an application')
  .option('--cwd <path>', 'Specify the project folder')
  .option('--id <id>', 'Specify the new instance ID')
  .action(async (id: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const app = cirno.get(id, 'backup')
    if (app.id !== id) error('Cannot backup a base instance.')
    const tar = new Tar()
    const tmp = join(cwd, 'tmp', id + '.baka')
    if (app.backups.length) {
      tar.loadFile(join(cwd, 'apps', id + '.baka'))
    }
    const meta = await loadMeta(join(cwd, 'apps', id))
    const newId = cirno.createId(options.id)
    cirno.state[app.id][newId] = meta
    app.backups.push({
      id: newId,
      type: 'manual',
      created: new Date().toISOString(),
    })
    tar.loadDir(join(cwd, 'apps', id), '/' + newId + '/')
    tar.dumpFile(tmp)
    await tar.finalize()
    await rename(tmp, join(cwd, 'apps', id + '.baka'))
    await cirno.save()
    success(`Successfully created a backup instance ${newId}.`)
  })
