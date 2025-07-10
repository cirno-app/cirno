import { CAC } from 'cac'
import { join, resolve } from 'node:path'
import { Cirno } from '../index.ts'
import { success } from '../utils.ts'

export default (cli: CAC) => cli
  .command('clone [id] [name]', 'Clone an instance')
  .option('--cwd <path>', 'Specify the project folder')
  .option('--id <id>', 'Specify the new instance ID')
  .action(async (id: string, name: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const app = cirno.get(id, 'clone')
    const newId = cirno.createId(options.id)
    cirno.apps[newId] = {
      id: newId,
      name: name ?? app.name,
      created: new Date().toISOString(),
      backups: [],
    }
    cirno.state[newId] = {}
    await cirno.clone(app, id, join(cwd, 'apps', newId))
    await cirno.save()
    success(`Successfully created a cloned instance ${newId}.`)
  })
