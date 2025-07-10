import { CAC } from 'cac'
import { join, resolve } from 'node:path'
import { Cirno, loadMeta } from '../index.ts'
import { error, loadIntoZip, success } from '../utils.ts'
import { ZipFS } from '@yarnpkg/libzip'
import { readFile, writeFile } from 'node:fs/promises'

export default (cli: CAC) => cli
  .command('backup [id]', 'Backup an application')
  .option('--cwd <path>', 'Specify the project folder')
  .option('--id <id>', 'Specify the new instance ID')
  .action(async (id: string, options) => {
    const cwd = resolve(process.cwd(), options.cwd ?? '.')
    const cirno = await Cirno.init(cwd)
    const app = cirno.get(id, 'backup')
    if (app.id !== id) error('Cannot backup a base instance.')
    const zip = new ZipFS(app.backups.length
      ? await readFile(join(cwd, 'apps', id + '.bak.zip'))
      : null)
    const meta = await loadMeta(join(cwd, 'apps', id))
    const newId = cirno.createId(options.id)
    cirno.state[app.id][newId] = meta
    app.backups.push({
      id: newId,
      type: 'manual',
      created: new Date().toISOString(),
    })
    await loadIntoZip(zip, join(cwd, 'apps', id), '/' + newId + '/')
    await writeFile(join(cwd, 'apps', id + '.bak.zip'), zip.getBufferAndClose())
    await cirno.save()
    success(`Successfully created a backup instance ${newId}.`)
  })
