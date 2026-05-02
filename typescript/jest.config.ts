// typescript/jest.config.ts
import type { Config } from 'jest'

const config: Config = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  testMatch: [
    '**/tests/**/*.test.ts',   // integration-style
    '**/src/**/*.test.ts'      // unit-style
  ],
  collectCoverage: true,
  coverageDirectory: 'build/coverage', // output under typescript/build/
  coverageReporters: ['lcov'],
  rootDir: '.',                        // base is typescript/
  moduleDirectories: ['node_modules', 'src'], // allow absolute imports from src/
}

export default config
