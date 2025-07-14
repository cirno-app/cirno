import { makeEnv, useFixture } from '../shared'

makeEnv('patch', (ctx) => {
  ctx.pass(['init'])
  const uuid1 = ctx.pass(['import', useFixture('patch-1')])
  const uuid2 = ctx.pass(['import', useFixture('patch-2')])
  ctx.pass(['remove', uuid1])
  ctx.pass(['remove', uuid2])
})
