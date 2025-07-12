import { PortablePath } from '@yarnpkg/fslib'
import { ZipFS } from '@yarnpkg/libzip'
import { join } from 'node:path'
import * as tarFs from 'tar-fs'
import * as tarStream from 'tar-stream'
import * as fs from 'node:fs/promises'
import k from 'kleur'
import { createReadStream, createWriteStream } from 'node:fs'
import { createBrotliCompress, createBrotliDecompress } from 'node:zlib'
import { finished } from 'node:stream/promises'
import { Writable } from 'node:stream'

export function info(message: string): undefined {
  console.log(k.bold(k.bgBlue(' INFO ') + ' ' + k.white(message)))
}

export function error(message: string, error?: any): never {
  console.log(k.bold(k.bgRed(' ERROR ') + ' ' + k.white(message)))
  if (error) console.error(error)
  process.exit(1)
}

export function success(message: string): never {
  console.log(k.bold(k.bgGreen(' SUCCESS ') + ' ' + k.white(message)))
  process.exit(0)
}

export async function dumpFromZip(zip: ZipFS, root: string, base = '/') {
  const [dirents] = await Promise.all([
    zip.readdirPromise(base as PortablePath, { withFileTypes: true }),
    fs.mkdir(root),
  ])
  await Promise.all(dirents.map(async (dirent) => {
    if (dirent.isFile()) {
      const buffer = await zip.readFilePromise(base + dirent.name as PortablePath)
      await fs.writeFile(join(root, dirent.name), buffer)
    } else if (dirent.isDirectory()) {
      await dumpFromZip(zip, join(root, dirent.name), base + dirent.name + '/')
    } else {
      throw new Error(`Unsupported file type`)
    }
  }))
}

export async function loadIntoZip(zip: ZipFS, root: string, base = '/') {
  const [dirents] = await Promise.all([
    fs.readdir(root, { withFileTypes: true }),
    base === '/' ? undefined : zip.mkdirPromise(base as PortablePath, { recursive: true }),
  ])
  await Promise.all(dirents.map(async (dirent) => {
    if (dirent.isFile()) {
      const buffer = await fs.readFile(join(root, dirent.name))
      await zip.writeFilePromise(base + dirent.name as PortablePath, buffer)
    } else if (dirent.isDirectory()) {
      await loadIntoZip(zip, join(root, dirent.name), base + dirent.name + '/')
    } else {
      throw new Error(`Unsupported file type`)
    }
  }))
}

export class Tar {
  private packs = [tarStream.pack()]
  private readTasks: (() => Promise<void>)[] = []
  private writeStreams: Writable[] = []

  createPack() {
    const pack = tarStream.pack()
    this.packs.push(pack)
    return pack
  }

  loadFile(root: string, filter: (header: tarStream.Headers, callback: (error?: unknown) => void) => boolean | tarStream.Pack = () => true) {
    this.readTasks.push(async () => {
      const extract = createReadStream(root)
        .pipe(createBrotliDecompress())
        .pipe(tarStream.extract())
      extract.on('entry', (header, stream, callback) => {
        const result = filter(header, callback)
        if (result === false) {
          stream.resume()
          callback()
        } else if (result === true) {
          stream.pipe(this.packs[0].entry(header, callback))
        } else {
          stream.pipe(result.entry(header, callback))
        }
      })
      await finished(extract)
    })
  }

  loadDir(root: string, base = '/') {
    base = base.slice(1)
    this.readTasks.push(async () => {
      const extract = tarStream.extract()
      extract.on('entry', (header, stream, callback) => {
        stream.pipe(this.packs[0].entry(header, callback))
      })
      tarFs.pack(root, {
        map: (header) => {
          header.name = join(base, header.name)
          return header
        },
      }).pipe(extract)
      await finished(extract)
    })
  }

  dumpFile(root: string, pack = this.packs[0]) {
    const stream = pack
      .pipe(createBrotliCompress())
      .pipe(createWriteStream(root))
    this.writeStreams.push(stream)
  }

  dumpDir(root: string, strip = 0, pack = this.packs[0]) {
    const stream = pack.pipe(tarFs.extract(root, { strip }))
    this.writeStreams.push(stream)
  }

  async finalize() {
    await Promise.all(this.readTasks.map(task => task()))
    await Promise.all(this.packs.map(pack => pack.finalize()))
    await Promise.all(this.writeStreams.map(stream => finished(stream)))
  }
}
