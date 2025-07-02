import { rm } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import type { GlobalSetupContext } from 'vitest/node'

const root = fileURLToPath(new URL('../temp', import.meta.url))

export async function setup(ctx: GlobalSetupContext) {
  await rm(root, { recursive: true, force: true })
}

export async function teardown(ctx: GlobalSetupContext) {
  await rm(root, { recursive: true, force: true })
}
