import { makeEnv, useExport, useFixture } from '../shared'

makeEnv('export', (ctx) => {
  ctx.pass(['init'])
  const uuid1 = ctx.pass(['import', useFixture('bar')])
  ctx.pass(['export', uuid1, useExport('bar.zip')])
  ctx.pass(['remove', uuid1])
  ctx.pass(['import', useExport('bar.zip')])
})
