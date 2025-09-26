module.exports = {
  testEnvironment: 'node',
  testMatch: [
    '**/tests/**/*.test.js'
  ],
  collectCoverageFrom: [
    '../index.js'
  ],
  coverageDirectory: 'coverage',
  coverageReporters: ['text', 'lcov', 'html'],
  testTimeout: 120000, // 2 minutes for stress tests
  setupFilesAfterEnv: ['<rootDir>/tests/setup.js'],
  verbose: true,
  maxWorkers: 4, // Limit workers for stress tests
};