#!/usr/bin/env node
/**
 * # Node.js Smoke Test Framework
 * 
 * Deadline-driven, cancellation-aware testing that catches big problems quickly.
 * Tests the core behaviors: deadlines, cancellation, retries, streaming, and resource hygiene.
 */

import { fetch } from 'undici';
import { AbortController } from 'abort-controller';

class SmokeTestError extends Error {
    constructor(type, details) {
        super();
        this.type = type;
        this.details = details;
        this.name = 'SmokeTestError';
    }
}

class RetryConfig {
    constructor(options = {}) {
        this.maxAttempts = options.maxAttempts || 3;
        this.backoffBase = options.backoffBase || 2.0;
        this.maxBackoffMs = options.maxBackoffMs || 5000;
        this.jitter = options.jitter !== false;
    }
}

class TimeoutConfig {
    constructor(options = {}) {
        this.connectMs = options.connectMs || 2000;
        this.tlsMs = options.tlsMs || 2000;
        this.readMs = options.readMs || 8000;
    }
}

class SmokeTestConfig {
    constructor(options = {}) {
        this.baseUrl = options.baseUrl || 'http://localhost:3000';
        this.model = options.model || 'test-model';
        this.deadlineMs = options.deadlineMs || 10000;
        this.timeouts = new TimeoutConfig(options.timeouts);
        this.retry = new RetryConfig(options.retry);
        this.idempotencyKey = options.idempotencyKey || this.generateIdempotencyKey();
    }

    generateIdempotencyKey() {
        return `test-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    }
}

class SmokeTestClient {
    constructor(config) {
        this.config = config;
    }

    async chatCompletion(messages, signal = null) {
        const startTime = Date.now();
        const deadline = startTime + this.config.deadlineMs;
        let attempt = 0;
        let lastError = null;

        while (attempt < this.config.retry.maxAttempts) {
            attempt++;
            const attemptStart = Date.now();

            // Check if we've exceeded the deadline
            if (attemptStart > deadline) {
                throw new SmokeTestError('Timeout', {
                    phase: 'deadline_exceeded',
                    elapsedMs: attemptStart - startTime,
                    remainingBudgetMs: 0
                });
            }

            // Calculate remaining budget for this attempt
            const remainingBudget = deadline - attemptStart;
            const timeoutMs = Math.min(remainingBudget, this.config.timeouts.readMs);

            try {
                const response = await this.makeRequestWithCancellation(messages, timeoutMs, signal);
                const data = await response.json();
                return data;
            } catch (error) {
                lastError = error;

                if (error.type === 'Canceled') {
                    throw new SmokeTestError('Canceled', {
                        phase: `attempt_${attempt}`,
                        elapsedMs: Date.now() - attemptStart
                    });
                }

                if (error.type === 'Timeout') {
                    lastError = new SmokeTestError('Timeout', {
                        phase: `attempt_${attempt}`,
                        elapsedMs: Date.now() - attemptStart,
                        remainingBudgetMs: deadline - Date.now()
                    });
                }

                if (error.type === 'RateLimited') {
                    const retryAfterMs = error.details.retryAfterSecs * 1000;
                    if (attemptStart + retryAfterMs > deadline) {
                        throw new SmokeTestError('RateLimited', {
                            retryAfterSecs: error.details.retryAfterSecs,
                            elapsedMs: Date.now() - attemptStart
                        });
                    }
                }

                // Non-retriable errors
                if (error.type === 'BadRequest' || error.type === 'ConnectionFailed') {
                    throw error;
                }

                // Calculate backoff for retry
                if (attempt < this.config.retry.maxAttempts) {
                    const backoffMs = this.calculateBackoff(attempt);
                    const backoffEnd = attemptStart + backoffMs;
                    
                    if (backoffEnd > deadline) {
                        break;
                    }

                    await this.sleep(backoffMs);
                }
            }
        }

        throw lastError || new SmokeTestError('Unexpected', {
            message: 'Max attempts exceeded',
            elapsedMs: Date.now() - startTime
        });
    }

    async makeRequestWithCancellation(messages, timeoutMs, signal) {
        const url = `${this.config.baseUrl}/v1/chat/completions`;
        const body = JSON.stringify({
            model: this.config.model,
            messages,
            max_tokens: 50
        });

        const headers = {
            'Content-Type': 'application/json'
        };

        if (this.config.idempotencyKey) {
            headers['Idempotency-Key'] = this.config.idempotencyKey;
        }

        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), timeoutMs);

        // Set up cancellation if signal provided
        if (signal) {
            signal.addEventListener('abort', () => controller.abort());
        }

        try {
            const response = await fetch(url, {
                method: 'POST',
                headers,
                body,
                signal: controller.signal
            });

            clearTimeout(timeoutId);

            const status = response.status;

            if (status >= 200 && status < 300) {
                return response;
            }

            if (status >= 400 && status < 500) {
                if (status === 429) {
                    const retryAfter = response.headers.get('retry-after');
                    const retryAfterSecs = retryAfter ? parseInt(retryAfter, 10) : 1;
                    
                    throw new SmokeTestError('RateLimited', {
                        retryAfterSecs,
                        elapsedMs: 0
                    });
                } else {
                    throw new SmokeTestError('BadRequest', {
                        status,
                        elapsedMs: 0
                    });
                }
            }

            if (status >= 500) {
                throw new SmokeTestError('Server5xx', {
                    status,
                    elapsedMs: 0
                });
            }

            throw new SmokeTestError('Unexpected', {
                message: `Unexpected status: ${status}`,
                elapsedMs: 0
            });

        } catch (error) {
            clearTimeout(timeoutId);

            if (error.name === 'AbortError') {
                if (signal && signal.aborted) {
                    throw new SmokeTestError('Canceled', {
                        phase: 'request_cancelled',
                        elapsedMs: 0
                    });
                } else {
                    throw new SmokeTestError('Timeout', {
                        phase: 'request_timeout',
                        elapsedMs: timeoutMs,
                        remainingBudgetMs: 0
                    });
                }
            }

            if (error.code === 'ECONNREFUSED' || error.code === 'ENOTFOUND') {
                throw new SmokeTestError('ConnectionFailed', {
                    phase: 'connection_failed',
                    elapsedMs: 0
                });
            }

            throw new SmokeTestError('Unexpected', {
                message: error.message,
                elapsedMs: 0
            });
        }
    }

    calculateBackoff(attempt) {
        const baseDelay = Math.min(
            Math.pow(this.config.retry.backoffBase, attempt - 1) * 1000,
            this.config.retry.maxBackoffMs
        );

        if (this.config.retry.jitter) {
            const jitter = Math.random() * baseDelay / 2;
            return Math.floor(baseDelay + jitter);
        }

        return Math.floor(baseDelay);
    }

    sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
}

class SmokeTestSuite {
    constructor(config) {
        this.client = new SmokeTestClient(config);
    }

    async testCancelDuringDns() {
        console.log('🧪 Testing cancellation during DNS...');
        
        const controller = new AbortController();
        
        // Cancel immediately (simulating DNS phase)
        controller.abort();
        
        const messages = [{ role: 'user', content: 'Hello' }];
        
        try {
            await this.client.chatCompletion(messages, controller.signal);
            throw new Error('Expected cancellation, but got success');
        } catch (error) {
            if (error.type === 'Canceled') {
                console.log(`✅ Canceled during ${error.details.phase} in ${error.details.elapsedMs}ms`);
                return;
            }
            throw error;
        }
    }

    async testCancelDuringConnect() {
        console.log('🧪 Testing cancellation during connection...');
        
        const controller = new AbortController();
        
        // Cancel after a short delay (simulating connection phase)
        setTimeout(() => controller.abort(), 100);
        
        const messages = [{ role: 'user', content: 'Hello' }];
        
        try {
            await this.client.chatCompletion(messages, controller.signal);
            throw new Error('Expected cancellation, but got success');
        } catch (error) {
            if (error.type === 'Canceled') {
                console.log(`✅ Canceled during ${error.details.phase} in ${error.details.elapsedMs}ms`);
                return;
            }
            throw error;
        }
    }

    async testDeadlineExceeded() {
        console.log('🧪 Testing deadline exceeded...');
        
        const config = new SmokeTestConfig({
            deadlineMs: 100, // Very short deadline
            baseUrl: 'http://localhost:3000' // Assuming Mockoon with timeout endpoint
        });
        
        const shortDeadlineClient = new SmokeTestClient(config);
        const messages = [{ role: 'user', content: 'Hello' }];
        
        try {
            await shortDeadlineClient.chatCompletion(messages);
            throw new Error('Expected timeout, but got success');
        } catch (error) {
            if (error.type === 'Timeout') {
                console.log(`✅ Timeout in ${error.details.phase} (remaining: ${error.details.remainingBudgetMs}ms) - ${error.details.elapsedMs}ms`);
                return;
            }
            throw error;
        }
    }

    async testRateLimitRetryAfter() {
        console.log('🧪 Testing rate limit with Retry-After...');
        
        const config = new SmokeTestConfig({
            baseUrl: 'http://localhost:3000' // Assuming Mockoon with rate limit endpoint
        });
        
        const rateLimitClient = new SmokeTestClient(config);
        const messages = [{ role: 'user', content: 'Hello' }];
        
        try {
            await rateLimitClient.chatCompletion(messages);
            throw new Error('Expected rate limit, but got success');
        } catch (error) {
            if (error.type === 'RateLimited') {
                console.log(`✅ Rate limited with Retry-After: ${error.details.retryAfterSecs}s (elapsed: ${error.details.elapsedMs}ms)`);
                return;
            }
            throw error;
        }
    }

    async testServer5xx() {
        console.log('🧪 Testing server 5xx error...');
        
        const config = new SmokeTestConfig({
            baseUrl: 'http://localhost:3000' // Assuming Mockoon with error endpoint
        });
        
        const errorClient = new SmokeTestClient(config);
        const messages = [{ role: 'user', content: 'Hello' }];
        
        try {
            await errorClient.chatCompletion(messages);
            throw new Error('Expected server error, but got success');
        } catch (error) {
            if (error.type === 'Server5xx') {
                console.log(`✅ Server 5xx error: ${error.details.status} (elapsed: ${error.details.elapsedMs}ms)`);
                return;
            }
            throw error;
        }
    }

    async testSuccessfulRequest() {
        console.log('🧪 Testing successful request...');
        
        const messages = [{ role: 'user', content: 'Hello' }];
        
        try {
            const response = await this.client.chatCompletion(messages);
            console.log(`✅ Successful request: ${JSON.stringify(response).substring(0, 100)}...`);
            return;
        } catch (error) {
            throw new Error(`Expected success, but got: ${error.message}`);
        }
    }

    async runAllTests() {
        console.log('🚀 Running Node.js smoke test suite...');
        
        const tests = [
            ['Cancel during DNS', () => this.testCancelDuringDns()],
            ['Cancel during connect', () => this.testCancelDuringConnect()],
            ['Deadline exceeded', () => this.testDeadlineExceeded()],
            ['Rate limit with Retry-After', () => this.testRateLimitRetryAfter()],
            ['Server 5xx error', () => this.testServer5xx()],
            ['Successful request', () => this.testSuccessfulRequest()]
        ];

        const failedTests = [];

        for (const [testName, testFn] of tests) {
            try {
                await testFn();
                console.log(`✅ ${testName}: PASSED`);
            } catch (error) {
                console.log(`❌ ${testName}: FAILED - ${error.message}`);
                failedTests.push([testName, error.message]);
            }
        }

        if (failedTests.length === 0) {
            console.log('🎉 All Node.js smoke tests passed!');
            return;
        } else {
            const errorMsg = failedTests
                .map(([name, error]) => `${name}: ${error}`)
                .join(', ');
            throw new Error(`Smoke tests failed: ${errorMsg}`);
        }
    }
}

// CLI interface
async function main() {
    const config = new SmokeTestConfig();
    const suite = new SmokeTestSuite(config);
    
    try {
        await suite.runAllTests();
        process.exit(0);
    } catch (error) {
        console.error(`❌ Smoke test suite failed: ${error.message}`);
        process.exit(1);
    }
}

if (import.meta.url === `file://${process.argv[1]}`) {
    main().catch(console.error);
}

export { SmokeTestSuite, SmokeTestClient, SmokeTestConfig, SmokeTestError };




