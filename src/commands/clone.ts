import { CAC } from 'cac'
import { resolve } from 'node:path'
import { cp } from 'node:fs/promises'
import { Cirno } from '../index.ts'
import { success } from '../utils.ts'

export default (cli: CAC) => cli
  .command('clone [id] [name]', 'Clone an instance')
  // .usage('Create a new instance with the same configuration as the base instance.')
  .option('--cwd <path>', 'Specify the project folder')
  .option('--id <id>', 'Specify the new instance ID')
  .action(async (id: string, name: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const base = cirno.get(id, 'clone')
    if (!base) return
    const head = cirno.create(name ?? base.name, options.id)
    head.backup = undefined
    await cp(cwd + '/instances/' + id, cwd + '/instances/' + head.id, { recursive: true })
    await cirno.save()
    success(`Successfully created a cloned instance ${head.id}.`)
  })
