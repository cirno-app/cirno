import { CAC } from 'cac'
import { join, resolve } from 'node:path'
import { Cirno } from '../index.ts'
import { error, success } from '../utils.ts'
import { mkdir, rename, writeFile } from 'node:fs/promises'

const NAME_REGEX = /^(?:(?:@(?:[a-z0-9-*~][a-z0-9-*._~]*)?\/[a-z0-9-._~])|[a-z0-9-~])[a-z0-9-._~]*$/

export default (cli: CAC) => cli
  .command('create [name]', 'Create a new application')
  .alias('new')
  .option('--cwd <path>', 'Specify the root folder')
  .option('--manager <manager>', 'Specify the package manager')
  .action(async (name, options) => {
    if (!name) error('Missing application name. See `cirno create --help` for usage.')
    if (!NAME_REGEX.test(name)) error('Invalid application name. See `cirno create --help` for usage.')

    if (!options.manager) error('Missing package manager. See `cirno create --help` for usage.')
    const capture = /^yarn@(\d+\.\d+\.\d+)/.exec(options.manager)
    if (!capture) error(`Unsupported package manager: ${options.manager}.`)

    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    await cirno.downloadYarn(capture[1])

    const temp = cwd + '/tmp/' + Math.random().toString(36).slice(2, 10).padEnd(8, '0')
    await mkdir(temp)
    // create an empty yarn.lock file
    await writeFile(join(temp, 'yarn.lock'), '')
    await writeFile(join(temp, 'package.json'), JSON.stringify({
      name,
      version: '0.0.0',
      private: true,
      packageManager: options.manager,
    }, null, 2) + '\n')
    const code = await cirno.yarn(temp, options['--'])
    if (code !== 0) error(`Failed to install dependencies. Exit code: ${code}`)

    const id = cirno.createId()
    cirno.apps[id] = {
      id,
      name,
      created: new Date().toISOString(),
      backups: [],
    }
    cirno.state[id] = {}
    await rename(temp, join(cwd, 'apps', id))
    await cirno.save()
    success(`Successfully created a new application ${id}.`)
  })
