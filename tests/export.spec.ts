import { makeEnv, useExport, useFixture } from './shared'

makeEnv((ctx) => {
  ctx.pass(['init'])
  const uuid1 = ctx.pass(['import', useFixture('foo')])
  ctx.pass(['export', uuid1, useExport('foo.zip')])
  ctx.pass(['remove', uuid1])
  ctx.pass(['import', useExport('foo.zip')])
})
