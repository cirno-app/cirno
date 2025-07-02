import { CAC } from 'cac'
import { join, resolve } from 'node:path'
import { Cirno, Package } from '../index.ts'
import { fork } from 'node:child_process'
import { readFile } from 'node:fs/promises'

export default (cli: CAC) => cli
  .command('yarn [id]', 'Execute Yarn in an instance')
  .option('--cwd <path>', 'Specify the project folder')
  .action(async (id, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    cirno.get(id, 'yarn')
    const pkgMeta: Package = JSON.parse(await readFile(join(cwd, 'instances', id, '/package.json'), 'utf8'))
    const capture = /^yarn@(\d+\.\d+\.\d+)/.exec(pkgMeta.packageManager)
    if (!capture) throw new Error('Failed to detect yarn version.')
    const yarnPath = join(cwd, `.yarn/releases/yarn-${capture[1]}.cjs`)
    const child = fork(yarnPath, options['--'], {
      cwd: join(cwd, 'instances', id),
      stdio: 'inherit',
    })
    child.on('exit', (code) => {
      process.exit(code)
    })
  })
