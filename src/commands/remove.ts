import { CAC } from 'cac'
import { resolve } from 'node:path'
import { rm } from 'node:fs/promises'
import { Cirno } from '../index.ts'
import { success } from '../utils.ts'

export default (cli: CAC) => cli
  .command('remove [id]', 'Remove an instance')
  .alias('rm')
  .option('--cwd <path>', 'Specify the project folder')
  .action(async (id: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    let instance = cirno.get(id, 'remove')
    if (!instance) return
    await rm(cwd + '/instances/' + id, { recursive: true, force: true })
    delete cirno.instances[id]
    if (instance.backup) {
      for (const next of Object.values(cirno.instances)) {
        if (next.parent === id) {
          next.parent = instance.parent
          break
        }
      }
    } else {
      while (instance.parent) {
        instance = cirno.instances[instance.parent]
        await rm(cwd + '/instances/' + instance.id, { recursive: true, force: true })
        delete cirno.instances[instance.id]
      }
    }
    await cirno.save()
    success(`Instance ${id} is successfully removed.`)
  })
