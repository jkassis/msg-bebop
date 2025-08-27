// src/pkg/math.test.ts
import { add } from './math';

describe('add (unit)', () => {
  it('adds 1 + 2 to equal 3', () => {
    expect(add(1, 2)).toBe(3);
  });
});
