import { CAC } from 'cac'
import { resolve } from 'node:path'
import { Cirno } from '../index.ts'
import { success } from '../utils.ts'

export default (cli: CAC) => cli
  .command('init', 'Initialize a new project')
  .option('--cwd <path>', 'Specify the project folder')
  .option('-f, --force', 'Overwrite existing project')
  .action(async (options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd, true, options.force)
    await cirno.save()
    success(`Cirno project initialized at ${cwd}.`)
  })
