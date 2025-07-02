import { makeEnv } from '../shared'

makeEnv((ctx) => {
  ctx.pass([])
  ctx.pass(['init'])
  ctx.fail(['init'])
})
