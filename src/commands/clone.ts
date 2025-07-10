import { CAC } from 'cac'
import { join, resolve } from 'node:path'
import { Cirno } from '../index.ts'
import { success } from '../utils.ts'

export default (cli: CAC) => cli
  .command('clone [id]', 'Clone an instance')
  .option('--cwd <path>', 'Specify the project folder')
  .option('--id <id>', 'Specify the new instance ID')
  .option('--name <name>', 'Specify the new application name')
  .action(async (id: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const app = cirno.get(id, 'clone')
    const newId = cirno.createId(options.id)
    cirno.apps[newId] = {
      id: newId,
      name: options.name ?? app.name,
      created: new Date().toISOString(),
      backups: [],
    }
    cirno.state[newId] = {}
    await cirno.clone(app, id, join(cwd, 'apps', newId))
    await cirno.save()
    success(`Successfully created a cloned instance ${newId}.`)
  })
