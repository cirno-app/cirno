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

  static async init(cwd: string, create = false) {
    try {
      const content = await fs.readFile(cwd + '/cirno.yml', 'utf8')
      return new Cirno(cwd, yaml.load(content) as Manifest)
    } catch {
      if (!create) error('Use `cirno init` to create a new project.')
      await fs.mkdir(cwd + '/instances', { recursive: true })
      await fs.mkdir(cwd + '/.yarn/cache', { recursive: true })
      await fs.mkdir(cwd + '/.yarn/releases', { recursive: true })
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

  get(id: string, command: string) {
    if (!id) error(`Missing instance ID. See \`cirno ${command} --help\` for usage.`)
    if (!validate(id)) error(`Invalid instance ID. See \`cirno ${command} --help\` for usage.`)
    if (!this.data.instances[id]) error(`Instance ${id} not found.`)
    return this.data.instances[id]
  }

  async save() {
    await fs.writeFile(this.cwd + '/cirno.yml', yaml.dump(this))
  }
}
