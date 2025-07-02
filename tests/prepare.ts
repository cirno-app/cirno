import { execSync } from 'node:child_process'

execSync('yarn set version 4.3.1', {
  cwd: new URL('./fixtures/foo', import.meta.url),
  stdio: 'inherit',
})

execSync('yarn set version 4.2.2', {
  cwd: new URL('./fixtures/bar', import.meta.url),
  stdio: 'inherit',
})
