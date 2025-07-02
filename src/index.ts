import * as fs from 'node:fs/promises'
import * as yaml from 'js-yaml'
import { error } from './utils.ts'
import { v4, validate } from 'uuid'

export interface YarnRc {
  yarnPath?: string
  enableGlobalCache?: boolean
}

export interface YarnLock extends Record<string, YarnLock.Entry> {
  __metadata: {
    version: number
    cacheKey: string
  } & YarnLock.Entry
}

export namespace YarnLock {
  export interface Entry {
    version: string
    resolution: string
    dependencies: Record<string, string>
    checksum: string
    languageName: string
    linkType: string
  }
}

export interface Package {
  name: string
  packageManager: string
}

export interface Manifest {
  version: string
  instances: Instance[]
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
  public instances: Record<string, Instance> = Object.create(null)

  private constructor(public cwd: string, public data: Manifest) {
    this.instances = Object.fromEntries(data.instances.map((instance) => [instance.id, instance]))
  }

  static async init(cwd: string, create = false, force = false) {
    try {
      const content = await fs.readFile(cwd + '/cirno.yml', 'utf8')
      if (create && force) throw new Error()
      if (create) error('Project already exists. Use `cirno init -f` to overwrite.')
      return new Cirno(cwd, yaml.load(content) as Manifest)
    } catch {
      if (!create) error('Use `cirno init` to create a new project.')
      await fs.mkdir(cwd + '/temp', { recursive: true })
      await fs.mkdir(cwd + '/instances', { recursive: true })
      await fs.mkdir(cwd + '/.yarn/cache', { recursive: true })
      await fs.mkdir(cwd + '/.yarn/releases', { recursive: true })
      return new Cirno(cwd, { version: '1.0', instances: [] })
    }
  }

  create(name: string, id?: string): Instance {
    if (!id) {
      do {
        id = v4()
      } while (this.instances[id])
    }
    return this.instances[id] = {
      id,
      name,
      created: new Date().toISOString(),
      updated: new Date().toISOString(),
    }
  }

  * prepareRestore(head: Instance, id?: string) {
    while (head.parent) {
      yield head
      if (head.parent === id) return
      head = this.instances[head.parent]
    }
    error(`Instance ${id} is not an ancestor of ${head.id}.`)
  }

  get(id: string, command: string) {
    if (!id) error(`Missing instance ID. See \`cirno ${command} --help\` for usage.`)
    if (!validate(id)) error(`Invalid instance ID. See \`cirno ${command} --help\` for usage.`)
    if (!this.instances[id]) error(`Instance ${id} not found.`)
    return this.instances[id]
  }

  head(id: string, command: string) {
    const head = this.get(id, command)
    if (head.backup) error(`You can only ${command} a head instance, but instance ${id} is already a backup.`)
    return head
  }

  async save() {
    this.data.instances = Object.values(this.instances)
    await fs.writeFile(this.cwd + '/cirno.yml', yaml.dump(this.data))
  }
}
