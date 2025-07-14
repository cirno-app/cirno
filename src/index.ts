import * as fs from 'node:fs/promises'
import * as yaml from 'js-yaml'
import * as zlib from 'node:zlib'
import { error, info, Tar } from './utils.ts'
import { join } from 'node:path'
import { fork } from 'node:child_process'
import { promisify } from 'node:util'
import { parseSyml, stringifySyml } from '@yarnpkg/parsers'
import { Readable } from 'node:stream'
import { finished } from 'node:stream/promises'
import { extract } from 'tar-fs'
import { slugifyLocator, tryParseLocator } from './yarn.ts'

export interface YarnRc {
  cacheFolder?: string
  enableGlobalCache?: 'true' | 'false'
  nodeLinker?: 'node-modules' | 'pnp' | 'pnpm'
  npmRegistryServer?: string
  yarnPath?: string
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
  created: string
}

export interface Config {}

export interface Backup {
  id: string
  type?: string
  message?: string
  created: string
}

export interface Meta {
  pkg: Package
  yarnRc: YarnRc
  yarnLock: YarnLock
}

export async function loadMeta(cwd: string): Promise<Meta> {
  const pkg: Package = JSON.parse(await fs.readFile(cwd + '/package.json', 'utf8'))
  const yarnRc: YarnRc = parseSyml(await fs.readFile(cwd + '/.yarnrc.yml', 'utf8'))
  const yarnLock = parseSyml(await fs.readFile(cwd + '/yarn.lock', 'utf8')) as YarnLock
  return { pkg, yarnRc, yarnLock }
}

export function getCacheFiles(yarnLock: YarnLock) {
  return Object.entries(yarnLock).map(([key, value]) => {
    if (key === '__metadata') return
    const locator = tryParseLocator(value.resolution, true)
    if (!locator) throw new Error(`Failed to parse resolution: ${value.resolution}`)
    if (locator.reference.startsWith('workspace:')) return
    return slugifyLocator(locator)
  }).filter(Boolean) as string[]
}

const ENTRY_FILE = 'cirno.yml'
const STATE_FILE = 'cirno-baka.br'
const compress = promisify(zlib.brotliCompress)
const decompress = promisify(zlib.brotliDecompress)

export class Cirno {
  public apps: Record<string, App> = Object.create(null)

  private constructor(public cwd: string, public data: Manifest, public state: Record<string, Record<string, Meta>>) {
    for (const app of data.apps) {
      this.apps[app.id] = app
      for (const backup of app.backups) {
        this.apps[backup.id] = app
      }
    }
  }

  static async _init(cwd: string) {
    await fs.mkdir(cwd, { recursive: true })
    await fs.mkdir(cwd + '/apps')
    await fs.mkdir(cwd + '/baka')
    await fs.mkdir(cwd + '/home')
    await fs.mkdir(cwd + '/home/.yarn')
    await fs.mkdir(cwd + '/home/.yarn/cache')
    await fs.mkdir(cwd + '/home/.yarn/releases')
    await fs.mkdir(cwd + '/tmp')
    if (process.platform === 'win32') {
      await fs.mkdir(cwd + '/home/AppData')
      await fs.mkdir(cwd + '/home/AppData/Local')
      await fs.mkdir(cwd + '/home/AppData/Roaming')
    }
    await fs.writeFile(cwd + '/home/.yarnrc.yml', stringifySyml({
      enableTips: 'false',
      nodeLinker: 'pnp',
      pnpEnableEsmLoader: 'true',
    }))
  }

  static async init(cwd: string, create = false, force = false) {
    const files = await fs.readdir(cwd).catch<string[]>(() => [])
    if (create) {
      if (files.length) {
        if (!force) error('Target directory is not empty. Use `cirno init -f` to overwrite.')
        await Promise.all(files.map(file => fs.rm(join(cwd, file), { recursive: true, force: true })))
      }
      await this._init(cwd)
      return new Cirno(cwd, { version, config: {}, apps: [] }, {})
    }

    try {
      const manifest = yaml.load(await fs.readFile(join(cwd, ENTRY_FILE), 'utf8')) as Manifest
      const state = JSON.parse((await decompress(await fs.readFile(join(cwd, STATE_FILE)))).toString())
      if (manifest.version !== version) error(`Unsupported version: ${manifest.version}`)
      return new Cirno(cwd, manifest, state)
    } catch (e) {
      if (files.length) {
        error('Target directory is not a valid Cirno environment. Use `cirno init -f` to overwrite or choose another directory.')
      } else {
        error('Target directory is empty. Use `cirno init` to create a new environment.')
      }
    }
  }

  createId(id?: string) {
    if (id) return id
    do {
      id = Math.random().toString(36).slice(2, 10).padEnd(8, '0')
    } while (this.apps[id])
    return id
  }

  get(id: string, command: string) {
    if (!id) error(`Missing instance ID. See \`cirno ${command} --help\` for usage.`)
    if (!/[0-9a-f-]+/.test(id)) error(`Invalid instance ID. See \`cirno ${command} --help\` for usage.`)
    const app = this.apps[id]
    if (!app) error(`Instance ${id} not found.`)
    return app
  }

  async save() {
    this.data.apps = Object.entries(this.apps)
      .filter(([id, app]) => id === app.id)
      .map(([_, app]) => app)
    await fs.writeFile(join(this.cwd, ENTRY_FILE), yaml.dump(this.data))
    await fs.writeFile(join(this.cwd, STATE_FILE), await compress(Buffer.from(JSON.stringify(this.state))))
  }

  async clone(app: App, id: string, dest: string) {
    if (app.id === id) {
      await fs.cp(join(this.cwd, 'apps', id), dest, { recursive: true })
    } else {
      const tar = new Tar(join(this.cwd, 'baka', id + '.tar.br'))
      tar.load()
      tar.extract(dest, 1)
      await tar.finalize()
    }
  }

  async loadCache() {
    const files = await fs.readdir(join(this.cwd, 'home/.yarn/cache'))
    const cache: Record<string, Record<string, string>> = Object.create(null)
    for (const name of files) {
      const capture = /^(.+)-([0-9a-f]+)\.zip$/.exec(name)
      if (!capture) continue
      (cache[capture[2]] ??= {})[capture[1]] = name
    }
    return cache
  }

  async downloadYarn(version: string, registry?: string) {
    const dest = join(this.cwd, `home/.yarn/releases/yarn-${version}.cjs`)
    try {
      await fs.access(dest)
      return
    } catch {}
    if (!registry) {
      const globalRc = parseSyml(await fs.readFile(join(this.cwd, 'home/.yarnrc.yml'), 'utf8')) as YarnRc
      registry = globalRc.npmRegistryServer ?? 'https://registry.yarnpkg.com'
    }
    info(`Downloading yarn@${version} from ${registry}`)
    const response = await fetch(`${registry}/@yarnpkg/cli-dist/-/cli-dist-${version}.tgz`)
    const temp = join(this.cwd, 'tmp', `yarn-${version}`)
    try {
      await finished(Readable.fromWeb(response.body as any)
        .pipe(zlib.createGunzip())
        .pipe(extract(temp, { strip: 1 })))
      await fs.rename(join(temp, 'bin/yarn.js'), dest)
    } finally {
      await fs.rm(temp, { recursive: true, force: true })
    }
  }

  async yarn(cwd: string, args: string[]) {
    const pkgMeta: Package = JSON.parse(await fs.readFile(join(cwd, '/package.json'), 'utf8'))
    const capture = /^yarn@(\d+\.\d+\.\d+)/.exec(pkgMeta.packageManager)
    if (!capture) throw new Error('Failed to detect yarn version.')
    const env: Record<string, string | undefined> = { ...process.env }

    env.HOME = join(this.cwd, 'home')
    env.TEMP = join(this.cwd, 'tmp')
    env.TMP = join(this.cwd, 'tmp')
    env.TMPDIR = join(this.cwd, 'tmp')
    env.CIRNO_HOST_HOME = process.env.HOME
    env.CIRNO_HOST_TEMP = process.env.TEMP
    env.CIRNO_HOST_TMP = process.env.TMP
    env.CIRNO_HOST_TMPDIR = process.env.TMPDIR

    if (process.platform === 'win32') {
      env.APPDATA = join(this.cwd, 'home/AppData/Roaming')
      env.LOCALAPPDATA = join(this.cwd, 'home/AppData/Local')
      env.USERPROFILE = join(this.cwd, 'home')
      env.CIRNO_HOST_APPDATA = process.env.APPDATA
      env.CIRNO_HOST_LOCALAPPDATA = process.env.LOCALAPPDATA
      env.CIRNO_HOST_USERPROFILE = process.env.USERPROFILE
    }

    const yarnPath = join(this.cwd, `home/.yarn/releases/yarn-${capture[1]}.cjs`)
    env.YARN_YARN_PATH = yarnPath
    env.YARN_GLOBAL_FOLDER = join(this.cwd, 'home/.yarn')

    return new Promise<number | null>((resolve, reject) => {
      const child = fork(yarnPath, args, {
        cwd,
        env,
        stdio: 'inherit',
      })
      child.on('error', reject)
      child.on('exit', (code) => {
        resolve(code)
      })
    })
  }

  async gc() {
    const cache = await this.loadCache()
    const releases = new Set(await fs.readdir(join(this.cwd, 'home/.yarn/releases')))
    await Promise.all(Object.keys(this.state).map(async (id) => {
      const { pkg, yarnLock } = await loadMeta(join(this.cwd, 'apps', id))
      const capture = /^yarn@(\d+\.\d+\.\d+)/.exec(pkg.packageManager)
      if (capture) releases.delete(`yarn-${capture[1]}.cjs`)
      for (const prefix of getCacheFiles(yarnLock)) {
        delete cache[yarnLock.__metadata.cacheKey]?.[prefix]
      }
    }))
    for (const { pkg, yarnLock } of Object.values(this.state).map(x => Object.values(x)).flat()) {
      const capture = /^yarn@(\d+\.\d+\.\d+)/.exec(pkg.packageManager)
      if (capture) releases.delete(`yarn-${capture[1]}.cjs`)
      for (const prefix of getCacheFiles(yarnLock)) {
        delete cache[yarnLock.__metadata.cacheKey]?.[prefix]
      }
    }
    for (const name of releases) {
      await fs.rm(join(this.cwd, 'home/.yarn/releases', name))
    }
    for (const prefixes of Object.values(cache)) {
      for (const name of Object.values(prefixes)) {
        await fs.rm(join(this.cwd, 'home/.yarn/cache', name))
      }
    }
  }
}
