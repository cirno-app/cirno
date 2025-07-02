import { PortablePath } from '@yarnpkg/fslib'
import { ZipFS } from '@yarnpkg/libzip'
import { join } from 'node:path'
import * as fs from 'node:fs/promises'
import k from 'kleur'

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
  const dirents = await zip.readdirPromise(base as PortablePath, { withFileTypes: true })
  await Promise.all(dirents.map(async (dirent) => {
    if (dirent.isFile()) {
      const buffer = await zip.readFilePromise(base + dirent.name as PortablePath)
      await fs.writeFile(join(root, dirent.name), buffer)
    } else if (dirent.isDirectory()) {
      await fs.mkdir(join(root, dirent.name))
      await dumpFromZip(zip, join(root, dirent.name), base + dirent.name + '/')
    } else {
      throw new Error(`Unsupported file type`)
    }
  }))
}

export async function loadIntoZip(zip: ZipFS, root: string, base = '/') {
  const dirents = await fs.readdir(root, { withFileTypes: true })
  await Promise.all(dirents.map(async (dirent) => {
    if (dirent.isFile()) {
      const buffer = await fs.readFile(join(root, dirent.name))
      await zip.writeFilePromise(base + dirent.name as PortablePath, buffer)
    } else if (dirent.isDirectory()) {
      await zip.mkdirPromise(base + dirent.name as PortablePath)
      await loadIntoZip(zip, join(root, dirent.name), base + dirent.name + '/')
    } else {
      throw new Error(`Unsupported file type`)
    }
  }))
}
