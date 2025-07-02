import { execSync } from 'node:child_process'
import { readdir } from 'node:fs/promises'

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

const dirents = await readdir(new URL('./fixtures', import.meta.url), { withFileTypes: true })
for (const dirent of dirents) {
  if (!dirent.isDirectory()) continue
  prepare(new URL(`./fixtures/${dirent.name}`, import.meta.url))
}
