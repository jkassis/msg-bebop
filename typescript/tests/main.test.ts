// typescript/tests/main.test.ts
import { add } from '../src/pkg/math'

describe('add', () => {
  it('adds two numbers', () => {
    expect(add(2, 3)).toBe(5)
  })
})
