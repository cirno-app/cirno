import { makeEnv, useFixture } from '../shared'

makeEnv('cache', (ctx) => {
  ctx.pass(['init'])
  const uuid1 = ctx.pass(['import', useFixture('base')])
  const uuid2 = ctx.pass(['import', useFixture('dep-1')])
  const uuid3 = ctx.pass(['import', useFixture('dep-2')])
  ctx.pass(['remove', uuid2])
  ctx.pass(['remove', uuid3])
})
