import { mkdir, readdir, readFile } from 'node:fs/promises'
import { fork } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { beforeAll, expect, it } from 'vitest'
import { v5 } from 'uuid'

const root = fileURLToPath(new URL('../temp', import.meta.url))

interface Output {
  stdout: string
  stderr: string
  code: number | null
  signal: string | null
  entries: Entry[]
}

interface Entry {
  type: 'file' | 'directory'
  name: string
  content?: string
  entries?: Entry[]
}

const ISO_REGEX = /\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z/gm

async function traverse(root: string) {
  const result: Entry[] = []
  const files = await readdir(root, { withFileTypes: true })
  for (const file of files) {
    if (file.isDirectory()) {
      const entry: Entry = { type: 'directory', name: file.name }
      if (!['index'].includes(file.name)) {
        entry.entries = await traverse(root + '/' + file.name)
      }
      result.push(entry)
    } else {
      const entry: Entry = { type: 'file', name: file.name }
      if (['cirno.yml', '.yarnrc.yml'].includes(file.name)) {
        const content = await readFile(root + '/' + file.name, 'utf8')
        entry.content = content.replace(ISO_REGEX, '<timestamp>')
      }
      result.push(entry)
    }
  }
  return result
}

function spawn(args: string[], root: string) {
  const cwd = root + '/data'
  return new Promise<Output>((resolve, reject) => {
    const child = fork(new URL('../src/cli.ts', import.meta.url), args, {
      cwd,
      execArgv: ['--import', 'tsx'],
      stdio: 'pipe',
      env: {
        ...process.env,
        // yarn will always display colors in CI environments
        GITHUB_ACTIONS: '',
      },
    })
    let stdout = '', stderr = ''
    child.stdout!.on('data', (data) => stdout += data)
    child.stderr!.on('data', (data) => stderr += data)
    child.on('exit', async (code, signal) => {
      stdout = stdout.replace(root, '').replace(/(Completed|Done) in ([\w ]+)/g, '$1')
      stderr = stderr.replace(root, '').replace(/(Completed|Done) in ([\w ]+)/g, '$1')
      const entries = await traverse(cwd)
      const output = { stdout, stderr, code, signal, entries }
      Object.defineProperty(output, '__SERIALIZER__', { value: 'yaml' })
      resolve(output)
    })
  })
}

const NS_ENV = '998fa86e-b6b0-48c6-99cc-bc982a2759c0'
const NS_SPEC = '226704d4-f5d0-4349-b8bb-d9d480b0433e'

let stepCount = 0
let instCount = 0

interface Arg {
  value: string
  pretty: string
}

export function useFixture(name: string): Arg {
  return { value: `../../../tests/fixtures/${name}`, pretty: `/fixtures/${name}` }
}

export function useExport(name: string): Arg {
  return { value: `../exports/${name}`, pretty: `/exports/${name}` }
}

export interface StepOptions {
  silent?: boolean
  code?: number
}

export function makeEnv(name: string, callback: (ctx: CirnoTestContext) => void) {
  const ctx = new CirnoTestContext(root + '/' + v5(name, NS_ENV).slice(0, 8))
  callback(ctx)
}

export class CirnoTestContext {
  constructor(public root: string) {
    beforeAll(async () => {
      await mkdir(root + '/data', { recursive: true })
      await mkdir(root + '/exports', { recursive: true })
    })
  }

  test(args: (string | Arg)[], options: StepOptions = {}): Arg {
    stepCount += 1
    const isCreate = ['import', 'clone', 'backup'].includes(args[0] as string)
    if (isCreate && !options.code) instCount += 1
    const uuid = v5(`${instCount}`, NS_SPEC).slice(0, 8)
    const name = [
      `step ${stepCount}:`,
      'cirno',
      ...args.map((arg) => typeof arg === 'string' ? arg : arg.pretty),
    ].join(' ')
    it(name, async () => {
      const output = await spawn([
        ...args.map((arg) => typeof arg === 'string' ? arg : arg.value),
        ...isCreate ? ['--id', uuid] : [],
      ], this.root)
      if (!options.silent) {
        expect(output).toMatchSnapshot()
      }
      if (options.code !== undefined) {
        expect(output.code).toBe(options.code)
      }
    })
    return { value: uuid, pretty: `<#${instCount}>` }
  }

  pass(args: (string | Arg)[], options: StepOptions = {}) {
    return this.test(args, { ...options, code: 0 })
  }

  fail(args: (string | Arg)[], options: StepOptions = {}) {
    return this.test(args, { ...options, code: 1 })
  }
}
