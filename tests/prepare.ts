import { execSync } from 'node:child_process'

function execute(cwd: URL, command: string) {
  execSync(command, { cwd, stdio: 'inherit' })
}

function prepare(cwd: URL) {
  execute(cwd, 'yarn set version self --yarn-path')
  execute(cwd, 'yarn config set enableGlobalCache false')
  execute(cwd, 'yarn config set enableTips false')
  execute(cwd, 'yarn config set nodeLinker pnp')
  execute(cwd, 'yarn')
}

prepare(new URL('./fixtures/foo', import.meta.url))
prepare(new URL('./fixtures/bar', import.meta.url))
