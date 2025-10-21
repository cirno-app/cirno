import { CAC } from 'cac'
import { resolve } from 'node:path'
import { Cirno } from '../index.ts'
import { success } from '../utils.ts'

export default (cli: CAC) => cli
  .command('init', 'Initialize a new environment')
  .option('--cwd <path>', 'Specify the root folder')
  .option('-f, --force', 'Overwrite existing environment')
  .action(async (options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd, true, options.force)
    await cirno.save()
    success(`Cirno environment initialized at ${cwd}.`)
  })
