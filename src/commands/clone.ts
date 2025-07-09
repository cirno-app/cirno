import { CAC } from 'cac'
import { resolve } from 'node:path'
import { cp } from 'node:fs/promises'
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
    cirno.instances[newId] = {
      id: newId,
      name: name ?? app.name,
      backups: [],
    }
    await cp(cwd + '/apps/' + id, cwd + '/apps/' + newId, { recursive: true })
    await cirno.save()
    success(`Successfully created a cloned instance ${newId}.`)
  })
