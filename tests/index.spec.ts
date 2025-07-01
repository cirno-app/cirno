import { mkdir, readdir, readFile, rm } from 'node:fs/promises'
import { fork } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { afterAll, beforeAll, expect, it } from 'vitest'
import { v5 } from 'uuid'

const cwd = fileURLToPath(new URL('../temp', import.meta.url))

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
  files: File[]
}

interface File {
  path: string
  content: string
}

async function traverse(root: string, prefix = '') {
  const result: File[] = []
  const files = await readdir(root, { withFileTypes: true })
  for (const file of files) {
    if (file.isDirectory()) {
      result.push(...await traverse(root + '/' + file.name, prefix + file.name + '/'))
    } else {
      result.push({
        path: prefix + file.name,
        content: await readFile(root + '/' + file.name, 'utf8'),
      })
    }
  }
  return result
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
    child.on('exit', async (code, signal) => {
      const files = await traverse(cwd)
      resolve({ stdout, stderr, code, signal, files })
    })
  })
}

const namespace = '226704d4-f5d0-4349-b8bb-d9d480b0433e'

const counters: Record<string, number> = Object.create(null)

function makeTest(args: string[], create?: boolean) {
  const name = ['cirno', ...args].join(' ')
  const counter = counters[name] = (counters[name] || 0) + 1
  const uuid = v5(`${name} ${counter}`, namespace)
  it(name, async () => {
    const output = await spawn([...args, ...create ? ['--id', uuid] : []])
    expect(output).toMatchSnapshot()
  })
  return uuid
}

makeTest([])
makeTest(['init'])
makeTest(['init'])
const uuid = makeTest(['import', '../tests/fixtures/foo'], true)
makeTest(['remove', uuid])
makeTest(['remove', uuid])
