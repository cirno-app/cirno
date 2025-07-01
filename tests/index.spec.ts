import { mkdir, readdir, readFile, rm } from 'node:fs/promises'
import { fork } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { afterAll, beforeAll, expect, it } from 'vitest'
import { v5 } from 'uuid'

const temp = fileURLToPath(new URL('../temp', import.meta.url))
const cwd = temp + '/data'
const out = temp + '/export'

beforeAll(async () => {
  await rm(temp, { recursive: true, force: true })
  await mkdir(cwd, { recursive: true })
})

afterAll(async () => {
  await rm(temp, { recursive: true, force: true })
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

const ISO_REGEX = /\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z/gm

async function traverse(root: string, prefix = '') {
  const result: File[] = []
  const files = await readdir(root, { withFileTypes: true })
  for (const file of files) {
    if (file.isDirectory()) {
      result.push(...await traverse(root + '/' + file.name, prefix + file.name + '/'))
    } else {
      const content = await readFile(root + '/' + file.name, 'utf8')
      result.push({
        path: prefix + file.name,
        content: content.replace(ISO_REGEX, '<timestamp>'),
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
      stdout = stdout.replace(out, '<export>')
      stderr = stderr.replace(out, '<export>')
      const files = await traverse(cwd)
      resolve({ stdout, stderr, code, signal, files })
    })
  })
}

const namespace = '226704d4-f5d0-4349-b8bb-d9d480b0433e'

let step = 0

function makeTest(args: string[], create?: boolean) {
  step += 1
  const uuid = v5(`${step}`, namespace)
  const name = [`step ${step.toString().padStart(2, '0')}:`, 'cirno', ...args].join(' ')
  it(name, async () => {
    const output = await spawn([...args, ...create ? ['--id', uuid] : []])
    expect(output).toMatchSnapshot()
  })
  return uuid
}

makeTest([])
makeTest(['init'])
makeTest(['init'])
const uuid1 = makeTest(['import', '../../tests/fixtures/foo'], true)
makeTest(['export', uuid1, '../export/foo.zip'])
makeTest(['remove', uuid1])
makeTest(['remove', uuid1])
const uuid2 = makeTest(['import', '../export/foo.zip'], true)
