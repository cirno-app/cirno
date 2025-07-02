import { makeEnv, makeTest, useExport, useFixture } from './shared'

makeEnv(() => {
  makeTest(['init'], { silent: true })
  const uuid1 = makeTest(['import', useFixture('foo')], { create: true })
  makeTest(['export', uuid1, useExport('foo.zip')])
  makeTest(['remove', uuid1])
  makeTest(['remove', uuid1])
  const uuid2 = makeTest(['import', useExport('foo.zip')], { create: true })
})
