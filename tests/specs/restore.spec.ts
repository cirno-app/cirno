import { makeEnv, useFixture } from '../shared'

makeEnv((ctx) => {
  ctx.pass(['init'])
  const uuid1 = ctx.pass(['import', useFixture('foo')])

  // create the first backup (#2 -> #1)
  const uuid2 = ctx.pass(['backup', uuid1])

  // restore can only be created from a head instance
  ctx.fail(['restore', uuid1])

  // create the second backup (#2 -> #3 -> #1)
  const uuid3 = ctx.pass(['backup', uuid1])

  // restore to the first backup, renamed to (#1)
  ctx.pass(['restore', uuid2])

  // the first backup is already renamed
  ctx.fail(['remove', uuid2])

  // the second backup is already deleted
  ctx.fail(['remove', uuid3])
})
