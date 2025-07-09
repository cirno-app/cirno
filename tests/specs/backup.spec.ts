import { makeEnv, useFixture } from '../shared'

makeEnv((ctx) => {
  ctx.pass(['init'])
  const uuid1 = ctx.pass(['import', useFixture('foo')])

  // create the first backup (#2 -> #1)
  const uuid2 = ctx.pass(['backup', uuid1])

  // backup can only be created from a head instance
  ctx.fail(['backup', uuid2])

  // create the second backup (#2 -> #3 -> #1)
  const uuid3 = ctx.pass(['backup', uuid1])

  // remove the first backup (#3 -> #1)
  ctx.pass(['remove', uuid2])

  // remove the head instance and its backups (empty)
  ctx.pass(['remove', uuid1, '--recursive'])

  // the second backup is already deleted
  ctx.fail(['remove', uuid3])
})
