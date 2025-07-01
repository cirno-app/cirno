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
  parent?: string
  backup?: Backup
}

export interface Backup {
  type: 'manual' | 'update' | 'scheduled'
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

  create(name: string): Instance {
    let id: string
    do {
      id = v4()
    } while (this.data.instances[id])
    return this.data.instances[id] = {
      id,
      name,
      created: new Date().toISOString(),
      updated: new Date().toISOString(),
    }
  }

  * prepareRestore(head: Instance, base: Instance) {
    while (head.parent) {
      yield head
      if (head.parent === base.id) return
      head = this.data.instances[head.parent]
    }
    error(`Instance ${base.id} is not an ancestor of ${head.id}.`)
  }

  get(id: string) {
    if (!id) error('Missing instance ID. See `cirno remove --help` for usage.')
    if (!validate(id)) error('Invalid instance ID. See `cirno remove --help` for usage.')
    if (!this.data.instances[id]) error(`Instance ${id} not found.`)
    return this.data.instances[id]
  }

  async save() {
    await fs.writeFile(this.cwd + '/cirno.yml', yaml.dump(this))
  }
}
