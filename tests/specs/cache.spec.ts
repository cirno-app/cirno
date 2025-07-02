import { makeEnv, useExport, useFixture } from '../shared'

makeEnv((ctx) => {
  ctx.pass(['init'])
  const uuid1 = ctx.pass(['import', useFixture('foo')])
  const uuid2 = ctx.pass(['import', useFixture('bar')])
  const uuid3 = ctx.pass(['import', useFixture('baz')])
})
