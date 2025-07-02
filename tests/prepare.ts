import { execSync } from 'node:child_process'

function execute(cwd: URL, command: string) {
  execSync(command, { cwd, stdio: 'inherit' })
}

const foo = new URL('./fixtures/foo', import.meta.url)
const bar = new URL('./fixtures/bar', import.meta.url)

execute(foo, 'yarn set version self --yarn-path')
execute(foo, 'yarn config set enableGlobalCache false')
execute(foo, 'yarn config set enableTips false')
execute(foo, 'yarn config set nodeLinker pnp')
execute(foo, 'yarn')

execute(bar, 'yarn set version self --yarn-path')
execute(bar, 'yarn config set enableGlobalCache false')
execute(bar, 'yarn config set enableTips false')
execute(bar, 'yarn config set nodeLinker pnp')
execute(bar, 'yarn')
