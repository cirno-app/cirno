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
  private callback?: () => Promise<void>
  private packs = [tarStream.pack()]
  private readables: Writable[] = []
  private writables: Writable[] = []

  constructor(public path: string) {}

  load(filter: (header: tarStream.Headers, callback: (error?: unknown) => void) => boolean | tarStream.Pack = () => true) {
    const extract = createReadStream(this.path)
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
    this.readables.push(extract)
  }

  pack(root: string, base = '') {
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
    this.readables.push(extract)
  }

  dump(temp: string, write = true) {
    if (write) {
      const stream = this.packs[0]
        .pipe(createBrotliCompress())
        .pipe(createWriteStream(temp))
      this.writables.push(stream)
      this.callback = async () => {
        await fs.rename(temp, this.path)
      }
    } else {
      this.callback = async () => {
        await fs.rm(this.path)
      }
    }
  }

  extract(root: string, strip = 0) {
    const pack = tarStream.pack()
    this.packs.push(pack)
    const stream = pack.pipe(tarFs.extract(root, { strip }))
    this.writables.push(stream)
    return pack
  }

  async finalize() {
    await Promise.all(this.readables.map(stream => finished(stream)))
    await Promise.all(this.packs.map(pack => pack.finalize()))
    await Promise.all(this.writables.map(stream => finished(stream)))
    await this.callback?.()
  }
}
