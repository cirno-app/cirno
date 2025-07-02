import { makeEnv, makeTest } from './shared'

makeEnv(() => {
  makeTest([])
  makeTest(['init'])
  makeTest(['init'])
})
