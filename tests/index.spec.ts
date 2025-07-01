import { mkdir, readdir, readFile, rm } from 'node:fs/promises'
import { fork } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { afterAll, beforeAll, expect, it } from 'vitest'
import { v5 } from 'uuid'

const temp = fileURLToPath(new URL('../temp', import.meta.url))
const cwd = temp + '/data'
const out = temp + '/exports'

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
      stdout = stdout.replace(out, '<exports>')
      stderr = stderr.replace(out, '<exports>')
      const files = await traverse(cwd)
      resolve({ stdout, stderr, code, signal, files })
    })
  })
}

const namespace = '226704d4-f5d0-4349-b8bb-d9d480b0433e'

let stepCount = 0
let instCount = 0

interface Arg {
  value: string
  pretty: string
}

function useFixture(name: string): Arg {
  return { value: `../../tests/fixtures/${name}`, pretty: `<fixtures>/${name}` }
}

function useExport(name: string): Arg {
  return { value: `../exports/${name}`, pretty: `<exports>/${name}` }
}

function makeTest(args: (string | Arg)[], create?: boolean): Arg {
  stepCount += 1
  if (create) instCount += 1
  const uuid = v5(`${instCount}`, namespace)
  const name = [
    `step ${stepCount}:`,
    'cirno',
    ...args.map((arg) => typeof arg === 'string' ? arg : arg.pretty),
  ].join(' ')
  it(name, async () => {
    const output = await spawn([
      ...args.map((arg) => typeof arg === 'string' ? arg : arg.value),
      ...create ? ['--id', uuid] : [],
    ])
    expect(output).toMatchSnapshot()
  })
  return { value: uuid, pretty: `<#${instCount}>` }
}

makeTest([])
makeTest(['init'])
makeTest(['init'])
const uuid1 = makeTest(['import', useFixture('foo')], true)
makeTest(['export', uuid1, useExport('foo.zip')])
makeTest(['remove', uuid1])
makeTest(['remove', uuid1])
const uuid2 = makeTest(['import', useExport('foo.zip')], true)
