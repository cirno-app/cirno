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
  private pack = tarStream.pack()
  private tasks: (() => Promise<void>)[] = []

  async loadFile(root: string, filter: (header: tarStream.Headers) => boolean | Tar = () => true) {
    this.tasks.push(async () => {
      const extract = tarStream.extract()
      extract.on('entry', (header, stream, callback) => {
        console.log(1, header.name)
        const result = filter(header)
        if (result === false) {
          stream.resume()
        } else if (result === true) {
          stream.pipe(this.pack.entry(header, callback))
        } else {
          stream.pipe(result.pack.entry(header, callback))
        }
      })
      await new Promise<void>((resolve, reject) => {
        extract.on('finish', resolve)
        extract.on('error', reject)
        createReadStream(root)
          // .pipe(createBrotliDecompress())
          .pipe(extract)
      })
      // await finished(extract)
    })
  }

  async loadDir(root: string, base = '/') {
    base = base.slice(1)
    this.tasks.push(async () => {
      const extract = tarStream.extract()
      extract.on('entry', (header, stream, callback) => {
        console.log(2, header.name)
        stream.pipe(this.pack.entry(header, callback))
      })
      tarFs.pack(root, {
        map: (header) => {
          header.name = join(base, header.name)
          return header
        },
      }).pipe(extract)
      await finished(extract)
      // tarFs.pack(root, {
      //   pack: this.pack,
      //   finalize: false,
      //   map: (header) => {
      //     header.name = join(base, header.name)
      //     return header
      //   },
      // })
    })
  }

  async dumpFile(root: string) {
    const stream = this.pack
      // .pipe(createBrotliCompress())
      .pipe(createWriteStream(root))
    for (const task of this.tasks) {
      await task()
    }
    this.pack.finalize()
    await finished(stream)
  }

  async dumpDir(root: string, base = '/') {
    base = base.slice(1)
    const stream = this.pack.pipe(tarFs.extract(root, {
      filter: (_, header) => {
        if (header?.name.startsWith(base)) {
          header.name = header.name.slice(base.length)
          return true
        }
        return false
      },
    }))
    for (const task of this.tasks) {
      await task()
    }
    await finished(stream)
  }
}
