import { execSync } from 'node:child_process'
import { existsSync, readdirSync } from 'node:fs'

function execute(cwd: URL, command: string) {
  execSync(command, { cwd, stdio: 'inherit' })
}

function prepare(cwd: URL) {
  execute(cwd, 'yarn set version self --yarn-path')
  execute(cwd, 'yarn config set enableGlobalCache false')
  execute(cwd, 'yarn config set enableTelemetry false')
  execute(cwd, 'yarn config set enableTips false')
  execute(cwd, 'yarn config set nodeLinker pnp')
  execute(cwd, 'yarn')
}

const baseURL = new URL('./fixtures', import.meta.url)
const dirents = readdirSync(baseURL, { withFileTypes: true })
for (const dirent of dirents) {
  if (!dirent.isDirectory()) continue
  if (existsSync(new URL(`./${dirent.name}/.yarnrc.yml`, baseURL))) continue
  prepare(new URL(`./${dirent.name}`, baseURL))
}
