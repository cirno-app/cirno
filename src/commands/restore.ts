import { CAC } from 'cac'
import { resolve } from 'node:path'
import { rm } from 'node:fs/promises'
import { Cirno } from '../index.ts'
import { success } from '../utils.ts'

export default (cli: CAC) => cli
  .command('restore [head] [base]', 'Restore an instance')
  // .usage('Restore an instance into previous backup, deleting all intermediate instances.')
  .option('--cwd <path>', 'Specify the project folder')
  .action(async (headId: string, baseId: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const head = cirno.head(headId, 'restore')
    if (!head) return
    const base = cirno.get(baseId, 'restore')
    if (!base) return
    const instances = [...cirno.prepareRestore(head, baseId)]
    await Promise.all(instances.map(async (instance) => {
      await rm(cwd + '/instances/' + instance.id, { recursive: true, force: true })
      delete cirno.instances[instance.id]
    }))
    base.backup = undefined
    await cirno.save()
    success(`Instance ${headId} is successfully restored to ${baseId}.`)
  })
