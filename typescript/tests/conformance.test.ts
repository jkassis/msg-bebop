import path from 'node:path'
import { runConformanceFixture } from '../src/pkg/conformance'

describe('conformance fixture', () => {
  it('passes shared suite.v1 scenarios', () => {
    const fixturePath = path.resolve(__dirname, '..', '..', 'conformance', 'fixtures', 'suite.v1.json')
    runConformanceFixture(fixturePath)
  })
})
