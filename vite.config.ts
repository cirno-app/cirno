import { basename, dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    env: {
      NODE_ENV: 'test',
    },
    globalSetup: [
      fileURLToPath(new URL('./tests/setup.ts', import.meta.url)),
    ],
    snapshotSerializers: [
      fileURLToPath(new URL('./tests/serializer.ts', import.meta.url)),
    ],
    resolveSnapshotPath: (path, extension) => {
      return join(dirname(path), '../snapshots', `${basename(path)}${extension}`)
    },
  },
})
