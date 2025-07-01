import * as fs from 'node:fs/promises'
import * as yaml from 'js-yaml'
import { error } from './utils.ts'
import { v4, validate } from 'uuid'

export interface Manifest {
  version: string
  instances: Record<string, Instance>
}

export interface Instance {
  id: string
  name: string
  created: string
  updated: string
  backup?: Backup
}

export interface Backup {
  id: string
  type?: string
}

export class Cirno {
  private constructor(public cwd: string, public data: Manifest) {}

  static async init(cwd: string) {
    await fs.mkdir(cwd + '/instances', { recursive: true })
    try {
      const content = await fs.readFile(cwd + '/cirno.yml', 'utf8')
      return new Cirno(cwd, yaml.load(content) as Manifest)
    } catch {
      return new Cirno(cwd, { version: '1.0', instances: {} })
    }
  }

  create(name: string, backup?: Backup) {
    let id: string
    do {
      id = v4()
    } while (this.data.instances[id])
    return this.data.instances[id] = {
      id,
      name,
      backup,
      created: new Date().toISOString(),
      updated: new Date().toISOString(),
    }
  }

  get(id: string) {
    if (!id) return error('Missing instance ID. See `cirno remove --help` for usage.')
    if (!validate(id)) return error('Invalid instance ID. See `cirno remove --help` for usage.')
    if (!this.data.instances[id]) return error(`Instance ${id} not found.`)
    return this.data.instances[id]
  }

  async save() {
    await fs.writeFile(this.cwd + '/cirno.yml', yaml.dump(this))
  }
}
