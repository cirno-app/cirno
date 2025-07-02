import { CAC } from 'cac'
import { resolve } from 'node:path'
import { Cirno } from '../index.ts'

export default (cli: CAC) => cli
  .command('yarn [id] -- [...]', 'Execute Yarn in an instance')
  .option('--cwd <path>', 'Specify the project folder')
  .action(async (id, _, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    cirno.get(id, 'yarn')
    const code = await cirno.yarn(id, options['--'])
    process.exit(code)
  })
