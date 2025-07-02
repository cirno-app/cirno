import { mkdir, readdir, readFile } from 'node:fs/promises'
import { fork } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { beforeAll, expect, it } from 'vitest'
import { v4, v5 } from 'uuid'

const root = fileURLToPath(new URL('../temp', import.meta.url))

interface Output {
  stdout: string
  stderr: string
  code: number | null
  signal: string | null
  files: File[]
}

interface File {
  path: string
  content?: string
}

const ISO_REGEX = /\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z/gm

async function traverse(root: string, prefix = '') {
  const result: File[] = []
  const files = await readdir(root, { withFileTypes: true })
  for (const file of files) {
    if (file.isDirectory()) {
      result.push(...await traverse(root + '/' + file.name, prefix + file.name + '/'))
    } else {
      let content: string | undefined
      if (['cirno.yml', 'yarnrc.yml'].includes(file.name)) {
        content = await readFile(root + '/' + file.name, 'utf8')
        content = content.replace(ISO_REGEX, '<timestamp>')
      }
      result.push({
        path: prefix + file.name,
        content,
      })
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
    })
    let stdout = '', stderr = ''
    child.stdout!.on('data', (data) => stdout += data)
    child.stderr!.on('data', (data) => stderr += data)
    child.on('exit', async (code, signal) => {
      stdout = stdout.replace(root, '')
      stderr = stderr.replace(root, '')
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

export function useFixture(name: string): Arg {
  return { value: `../../../tests/fixtures/${name}`, pretty: `<fixtures>/${name}` }
}

export function useExport(name: string): Arg {
  return { value: `../exports/${name}`, pretty: `<exports>/${name}` }
}

export interface StepOptions {
  silent?: boolean
  code?: number
}

export function makeEnv(callback: (ctx: CirnoTestContext) => void) {
  const ctx = new CirnoTestContext(root + '/' + v4())
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
    if (isCreate) instCount += 1
    const uuid = v5(`${instCount}`, namespace)
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
