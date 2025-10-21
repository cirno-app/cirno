import { makeEnv } from '../shared'

makeEnv('init', (ctx) => {
  ctx.pass([])
  ctx.pass(['init'])
  ctx.fail(['init'])
})
