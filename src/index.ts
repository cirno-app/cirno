import * as fs from 'node:fs/promises'
import * as yaml from 'js-yaml'

export interface Cirno {
  version: string
  instances: Record<string, Instance>
}

export interface Instance {
  id: string
  name: string
  created: string
  updated: string
  backup?: {
    id: string
    type?: string
  }
}

export async function init(cwd: string) {
  await fs.mkdir(cwd + '/instances', { recursive: true })
  try {
    const content = await fs.readFile(cwd + '/cirno.yml', 'utf8')
    return yaml.load(content) as Cirno
  } catch {
    return { version: '1.0', instances: {} }
  }
}

export async function save(cwd: string, cirno: Cirno) {
  await fs.writeFile(cwd + '/cirno.yml', yaml.dump(cirno))
}
