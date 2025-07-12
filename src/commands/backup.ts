import { CAC } from 'cac'
import { join, resolve } from 'node:path'
import { Cirno, loadMeta } from '../index.ts'
import { error, success, Tar } from '../utils.ts'

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
    if (app.backups.length) {
      await tar.loadFile(join(cwd, 'apps', id + '.tar'))
    }
    const meta = await loadMeta(join(cwd, 'apps', id))
    const newId = cirno.createId(options.id)
    cirno.state[app.id][newId] = meta
    app.backups.push({
      id: newId,
      type: 'manual',
      created: new Date().toISOString(),
    })
    await tar.loadDir(join(cwd, 'apps', id), '/' + newId + '/')
    await tar.dumpFile(join(cwd, 'apps', id + '.tar'))
    await cirno.save()
    success(`Successfully created a backup instance ${newId}.`)
  })
