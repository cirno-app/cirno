import { makeEnv, useFixture } from '../shared'

makeEnv('list', (ctx) => {
  ctx.pass(['init'])
  const uuid1 = ctx.pass(['import', useFixture('base')])
  const uuid2 = ctx.pass(['clone', uuid1])
  const uuid3 = ctx.pass(['backup', uuid2])
  ctx.pass(['list'])
})
