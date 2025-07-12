import { CAC } from 'cac'
import { resolve } from 'node:path'
import { Cirno } from '../index.ts'

export default (cli: CAC) => cli
  .command('gc', 'Garbage collection')
  .alias('prune')
  .option('--cwd <path>', 'Specify the root folder')
  .action(async (options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    await cirno.gc()
  })
