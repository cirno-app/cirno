import { SnapshotSerializer } from 'vitest'
import * as yaml from 'js-yaml'

export default {
  serialize(value, config, indentation, depth, refs, printer) {
    return yaml.dump(value, { lineWidth: 160 })
  },
  test(value) {
    return value && value.__SERIALIZER__ === 'yaml'
  },
} satisfies SnapshotSerializer
