import { CAC } from 'cac'
import { resolve } from 'node:path'
import { cp } from 'node:fs/promises'
import { Cirno } from '../index.ts'
import { success } from '../utils.ts'

export default (cli: CAC) => cli
  .command('backup [id] [name]', 'Backup an instance')
  // .usage('Create a backup instance and link it to the base instance.')
  .option('--cwd <path>', 'Specify the project folder')
  .option('--id <id>', 'Specify the new instance ID')
  .action(async (id: string, name: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const head = cirno.head(id, 'backup')
    if (!head) return
    const base = cirno.create(name ?? head.name, options.id)
    base.backup = { type: 'manual' }
    base.parent = head.parent
    head.parent = base.id
    await cp(cwd + '/instances/' + id, cwd + '/instances/' + base.id, { recursive: true })
    await cirno.save()
    success(`Successfully created a backup instance ${base.id}.`)
  })
