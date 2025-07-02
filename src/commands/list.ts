import { CAC } from 'cac'
import { resolve } from 'node:path'
import { Cirno } from '../index.ts'
import { info } from '../utils.ts'

export default (cli: CAC) => cli
  .command('list', 'List all instances')
  .alias('ls')
  .option('--cwd <path>', 'Specify the project folder')
  .option('--json', 'Output as JSON')
  .action(async (options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const instances = Object.values(cirno.instances)
    if (options.json) return console.log(JSON.stringify(instances))
    if (!instances.length) return info('No instances found.')
    info(`Found ${instances.length} instances:`)
    for (const instance of instances) {
      console.log(`${instance.id}\t${instance.name}`)
    }
  })
