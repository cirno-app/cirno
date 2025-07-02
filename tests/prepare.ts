import { execSync } from 'node:child_process'

function execute(cwd: URL, command: string) {
  execSync(command, { cwd, stdio: 'inherit' })
}

const foo = new URL('./fixtures/foo', import.meta.url)
const bar = new URL('./fixtures/bar', import.meta.url)

execute(foo, 'yarn set version 4.3.1')
execute(foo, 'yarn config set enableGlobalCache false')
execute(foo, 'yarn')

execute(bar, 'yarn set version 4.2.2')
execute(bar, 'yarn config set enableGlobalCache false')
execute(bar, 'yarn')
