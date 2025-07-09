import { CAC } from 'cac'
import { resolve } from 'node:path'
import { cp } from 'node:fs/promises'
import { Cirno, loadMeta } from '../index.ts'
import { error, success } from '../utils.ts'

export default (cli: CAC) => cli
  .command('backup [id]', 'Backup an application')
  .option('--cwd <path>', 'Specify the project folder')
  .option('--id <id>', 'Specify the new instance ID')
  .action(async (id: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const app = cirno.get(id, 'backup')
    if (app.id !== id) error('Cannot backup a base instance.')
    const meta = await loadMeta(cwd + '/apps/' + id)
    const newId = cirno.createId(options.id)
    cirno.state[app.id][newId] = meta
    app.backups.push({
      id: newId,
      type: 'manual',
      created: new Date().toISOString(),
    })
    await cp(cwd + '/apps/' + id, cwd + '/apps/' + newId, { recursive: true })
    await cirno.save()
    success(`Successfully created a backup instance ${newId}.`)
  })
