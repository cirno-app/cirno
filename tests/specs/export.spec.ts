import { makeEnv, useExport, useFixture } from '../shared'

makeEnv('export', (ctx) => {
  ctx.pass(['init'])
  const uuid1 = ctx.pass(['import', useFixture('dep-1')])
  ctx.pass(['export', uuid1, useExport('dep-1.zip')])
  ctx.pass(['remove', uuid1])
  ctx.pass(['import', useExport('dep-1.zip')])
})
