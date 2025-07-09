import * as fs from 'node:fs/promises'
import * as yaml from 'js-yaml'
import { error } from './utils.ts'
import { join } from 'node:path'
import { fork } from 'node:child_process'

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

const version = '1.0'

export interface Manifest {
  version: string
  config: Config
  apps: App[]
}

export interface App {
  id: string
  name: string
  backups: Backup[]
}

export interface Config {}

export interface Backup {
  id: string
  type?: string
  message?: string
  createTime: string
}

export class Cirno {
  public instances: Record<string, App> = Object.create(null)

  private constructor(public cwd: string, public data: Manifest) {
    for (const project of data.apps) {
      this.instances[project.id] = project
      for (const backup of project.backups) {
        this.instances[backup.id] = project
      }
    }
  }

  static async init(cwd: string, create = false, force = false) {
    try {
      const content = await fs.readFile(cwd + '/cirno.yml', 'utf8')
      if (create && force) throw new Error()
      if (create) error('Project already exists. Use `cirno init -f` to overwrite.')
      const data = yaml.load(content) as Manifest
      if (data.version !== version) error(`Unsupported version: ${data.version}`)
      return new Cirno(cwd, data)
    } catch {
      if (!create) error('Use `cirno init` to create a new project.')
      await fs.rm(cwd, { recursive: true, force: true })
      await fs.mkdir(cwd + '/temp', { recursive: true })
      await fs.mkdir(cwd + '/apps', { recursive: true })
      await fs.mkdir(cwd + '/.yarn/cache', { recursive: true })
      await fs.mkdir(cwd + '/.yarn/releases', { recursive: true })
      return new Cirno(cwd, { version, config: {}, apps: [] })
    }
  }

  createId(id?: string) {
    if (id) return id
    do {
      id = Math.random().toString(36).slice(2, 10).padEnd(8, '0')
    } while (this.instances[id])
    return id
  }

  get(id: string, command: string) {
    if (!id) error(`Missing instance ID. See \`cirno ${command} --help\` for usage.`)
    if (!/[0-9a-f-]+/.test(id)) error(`Invalid instance ID. See \`cirno ${command} --help\` for usage.`)
    const app = this.instances[id]
    if (!app) error(`Instance ${id} not found.`)
    return app
  }

  async save() {
    this.data.apps = Object.entries(this.instances)
      .filter(([id, app]) => id === app.id)
      .map(([_, app]) => app)
    await fs.writeFile(this.cwd + '/cirno.yml', yaml.dump(this.data))
  }

  async yarn(id: string, args: string[]) {
    const pkgMeta: Package = JSON.parse(await fs.readFile(join(this.cwd, 'apps', id, '/package.json'), 'utf8'))
    const capture = /^yarn@(\d+\.\d+\.\d+)/.exec(pkgMeta.packageManager)
    if (!capture) throw new Error('Failed to detect yarn version.')
    const yarnPath = join(this.cwd, `.yarn/releases/yarn-${capture[1]}.cjs`)
    return new Promise<number | null>((resolve, reject) => {
      const child = fork(yarnPath, args, {
        cwd: join(this.cwd, 'apps', id),
        stdio: 'inherit',
        env: {
          ...process.env,
          YARN_YARN_PATH: yarnPath,
          YARN_GLOBAL_FOLDER: this.cwd + '/.yarn',
        },
      })
      child.on('error', reject)
      child.on('exit', (code) => {
        resolve(code)
      })
    })
  }
}
