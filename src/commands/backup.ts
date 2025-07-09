import { CAC } from 'cac'
import { resolve } from 'node:path'
import { cp } from 'node:fs/promises'
import { Cirno } from '../index.ts'
import { success } from '../utils.ts'

export default (cli: CAC) => cli
  .command('backup [id]', 'Backup an application')
  .option('--cwd <path>', 'Specify the project folder')
  .option('--id <id>', 'Specify the new instance ID')
  .action(async (id: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const app = cirno.getHead(id, 'backup')
    const newId = cirno.createId(options.id)
    app.backups.push({
      id: newId,
      type: 'manual',
      createTime: new Date().toISOString(),
    })
    await cp(cwd + '/instances/' + id, cwd + '/instances/' + newId, { recursive: true })
    await cirno.save()
    success(`Successfully created a backup instance ${newId}.`)
  })
