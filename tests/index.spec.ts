import { mkdir, rm } from 'node:fs/promises'
import { fork } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { afterAll, beforeAll, expect, it } from 'vitest'

const cwd = fileURLToPath(new URL('./temp', import.meta.url))

beforeAll(async () => {
  await mkdir(cwd, { recursive: true })
})

afterAll(async () => {
  await rm(cwd, { recursive: true, force: true })
})

interface Output {
  stdout: string
  stderr: string
  code: number | null
  signal: string | null
}

function spawn(args: string[]) {
  return new Promise<Output>((resolve, reject) => {
    const child = fork(new URL('../src/cli.ts', import.meta.url), args, {
      cwd,
      execArgv: ['--import', 'tsx'],
      stdio: 'pipe',
    })
    let stdout = '', stderr = ''
    child.stdout!.on('data', (data) => stdout += data)
    child.stderr!.on('data', (data) => stderr += data)
    child.on('exit', (code, signal) => {
      resolve({ stdout, stderr, code, signal })
    })
  })
}

it('cirno', async () => {
  const output = await spawn([])
  expect(output).toMatchSnapshot()
})

it('cirno init', async () => {
  const output = await spawn(['init'])
  expect(output).toMatchSnapshot()
})

it('cirno init', async () => {
  const output = await spawn(['init'])
  expect(output).toMatchSnapshot()
})
