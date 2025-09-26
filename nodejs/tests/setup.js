/**
 * Jest test setup for LightLLM Rust Node.js bindings
 */

// Extend Jest timeout for stress tests
jest.setTimeout(120000);

// Setup console formatting for better test output
const originalConsoleLog = console.log;
console.log = (...args) => {
  const timestamp = new Date().toISOString().substr(11, 8);
  originalConsoleLog(`[${timestamp}]`, ...args);
};

// Global error handler for unhandled promises
process.on('unhandledRejection', (reason, promise) => {
  console.error('Unhandled Rejection at:', promise, 'reason:', reason);
});

// Enable garbage collection for memory tests if available
if (typeof global.gc === 'function') {
  console.log('🧹 Garbage collection available for memory tests');
} else {
  console.log('ℹ️  Garbage collection not available (run with --expose-gc for memory tests)');
}

// Performance monitoring utilities
global.performanceUtils = {
  measureSync: (name, fn) => {
    const start = process.hrtime.bigint();
    const result = fn();
    const end = process.hrtime.bigint();
    const duration = Number(end - start) / 1000000; // Convert to milliseconds

    return {
      result,
      duration,
      name
    };
  },

  measureAsync: async (name, fn) => {
    const start = process.hrtime.bigint();
    const result = await fn();
    const end = process.hrtime.bigint();
    const duration = Number(end - start) / 1000000;

    return {
      result,
      duration,
      name
    };
  },

  getMemory: () => {
    const usage = process.memoryUsage();
    return {
      rss: usage.rss / 1024 / 1024,
      heapUsed: usage.heapUsed / 1024 / 1024,
      heapTotal: usage.heapTotal / 1024 / 1024,
      external: usage.external / 1024 / 1024
    };
  }
};

// Test categories for organization
global.testCategories = {
  BASIC: 'basic',
  PERFORMANCE: 'performance',
  STRESS: 'stress',
  ERROR_HANDLING: 'error_handling'
};

// Console formatting helpers
global.logSection = (title) => {
  console.log(`\n${'='.repeat(60)}`);
  console.log(`🧪 ${title}`);
  console.log(`${'='.repeat(60)}`);
};

global.logTest = (name) => {
  console.log(`\n🔬 ${name}`);
  console.log(`${'-'.repeat(40)}`);
};

global.logResult = (result, unit = '') => {
  const icon = result >= 1000 ? '🚀' : result >= 100 ? '⚡' : '✅';
  console.log(`${icon} Result: ${result.toFixed(2)}${unit}`);
};

// Before each test
beforeEach(() => {
  // Force garbage collection if available
  if (global.gc) {
    global.gc();
  }
});

// After each test
afterEach(() => {
  // Clean up any remaining resources
  if (global.gc) {
    global.gc();
  }
});