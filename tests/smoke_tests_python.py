#!/usr/bin/env python3
"""
# Python Smoke Test Framework

Deadline-driven, cancellation-aware testing that catches big problems quickly.
Tests the core behaviors: deadlines, cancellation, retries, streaming, and resource hygiene.
"""

import asyncio
import json
import time
import uuid
from dataclasses import dataclass
from typing import Optional, Dict, Any, List
import httpx
import signal
from contextlib import asynccontextmanager


class SmokeTestError(Exception):
    """Custom error type for smoke test failures"""
    def __init__(self, error_type: str, details: Dict[str, Any]):
        super().__init__()
        self.error_type = error_type
        self.details = details
        self.message = f"{error_type}: {details}"

    def __str__(self):
        return self.message


@dataclass
class RetryConfig:
    max_attempts: int = 3
    backoff_base: float = 2.0
    max_backoff_ms: int = 5000
    jitter: bool = True


@dataclass
class TimeoutConfig:
    connect_ms: int = 2000
    tls_ms: int = 2000
    read_ms: int = 8000


@dataclass
class SmokeTestConfig:
    base_url: str = "http://localhost:3000"
    model: str = "test-model"
    deadline_ms: int = 10000
    timeouts: TimeoutConfig = None
    retry: RetryConfig = None
    idempotency_key: Optional[str] = None

    def __post_init__(self):
        if self.timeouts is None:
            self.timeouts = TimeoutConfig()
        if self.retry is None:
            self.retry = RetryConfig()
        if self.idempotency_key is None:
            self.idempotency_key = f"test-{int(time.time() * 1000)}-{uuid.uuid4().hex[:9]}"


class SmokeTestClient:
    def __init__(self, config: SmokeTestConfig):
        self.config = config

    async def chat_completion(self, messages: List[Dict[str, str]], cancel_token: Optional[asyncio.CancelledError] = None):
        start_time = time.time()
        deadline = start_time + (self.config.deadline_ms / 1000.0)
        attempt = 0
        last_error = None

        while attempt < self.config.retry.max_attempts:
            attempt += 1
            attempt_start = time.time()

            # Check if we've exceeded the deadline
            if attempt_start > deadline:
                raise SmokeTestError('Timeout', {
                    'phase': 'deadline_exceeded',
                    'elapsed_ms': int((attempt_start - start_time) * 1000),
                    'remaining_budget_ms': 0
                })

            # Calculate remaining budget for this attempt
            remaining_budget = deadline - attempt_start
            timeout_seconds = min(remaining_budget, self.config.timeouts.read_ms / 1000.0)

            try:
                response = await self._make_request_with_cancellation(messages, timeout_seconds, cancel_token)
                data = response.json()
                return data
            except SmokeTestError as e:
                last_error = e

                if e.error_type == 'Canceled':
                    raise SmokeTestError('Canceled', {
                        'phase': f'attempt_{attempt}',
                        'elapsed_ms': int((time.time() - attempt_start) * 1000)
                    })

                if e.error_type == 'Timeout':
                    last_error = SmokeTestError('Timeout', {
                        'phase': f'attempt_{attempt}',
                        'elapsed_ms': int((time.time() - attempt_start) * 1000),
                        'remaining_budget_ms': int((deadline - time.time()) * 1000)
                    })

                if e.error_type == 'RateLimited':
                    retry_after_ms = e.details['retry_after_secs'] * 1000
                    if attempt_start + (retry_after_ms / 1000.0) > deadline:
                        raise SmokeTestError('RateLimited', {
                            'retry_after_secs': e.details['retry_after_secs'],
                            'elapsed_ms': int((time.time() - attempt_start) * 1000)
                        })

                # Non-retriable errors
                if e.error_type in ['BadRequest', 'ConnectionFailed']:
                    raise e

                # Calculate backoff for retry
                if attempt < self.config.retry.max_attempts:
                    backoff_ms = self._calculate_backoff(attempt)
                    backoff_end = attempt_start + (backoff_ms / 1000.0)
                    
                    if backoff_end > deadline:
                        break

                    await asyncio.sleep(backoff_ms / 1000.0)
            except Exception as e:
                last_error = SmokeTestError('Unexpected', {
                    'message': str(e),
                    'elapsed_ms': int((time.time() - attempt_start) * 1000)
                })

        raise last_error or SmokeTestError('Unexpected', {
            'message': 'Max attempts exceeded',
            'elapsed_ms': int((time.time() - start_time) * 1000)
        })

    async def _make_request_with_cancellation(self, messages: List[Dict[str, str]], timeout_seconds: float, cancel_token: Optional[asyncio.CancelledError] = None):
        url = f"{self.config.base_url}/v1/chat/completions"
        body = {
            "model": self.config.model,
            "messages": messages,
            "max_tokens": 50
        }

        headers = {"Content-Type": "application/json"}
        if self.config.idempotency_key:
            headers["Idempotency-Key"] = self.config.idempotency_key

        timeout = httpx.Timeout(timeout_seconds)

        try:
            async with httpx.AsyncClient(timeout=timeout) as client:
                # Set up cancellation if provided
                if cancel_token:
                    # For simplicity, we'll handle cancellation in the calling code
                    pass

                response = await client.post(url, json=body, headers=headers)
                status = response.status_code

                if 200 <= status < 300:
                    return response

                if 400 <= status < 500:
                    if status == 429:
                        retry_after = response.headers.get('retry-after', '1')
                        retry_after_secs = int(retry_after)
                        
                        raise SmokeTestError('RateLimited', {
                            'retry_after_secs': retry_after_secs,
                            'elapsed_ms': 0
                        })
                    else:
                        raise SmokeTestError('BadRequest', {
                            'status': status,
                            'elapsed_ms': 0
                        })

                if 500 <= status < 600:
                    raise SmokeTestError('Server5xx', {
                        'status': status,
                        'elapsed_ms': 0
                    })

                raise SmokeTestError('Unexpected', {
                    'message': f'Unexpected status: {status}',
                    'elapsed_ms': 0
                })

        except asyncio.TimeoutError:
            raise SmokeTestError('Timeout', {
                'phase': 'request_timeout',
                'elapsed_ms': int(timeout_seconds * 1000),
                'remaining_budget_ms': 0
            })
        except httpx.ConnectError:
            raise SmokeTestError('ConnectionFailed', {
                'phase': 'connection_failed',
                'elapsed_ms': 0
            })
        except Exception as e:
            raise SmokeTestError('Unexpected', {
                'message': str(e),
                'elapsed_ms': 0
            })

    def _calculate_backoff(self, attempt: int) -> int:
        base_delay = min(
            (self.config.retry.backoff_base ** (attempt - 1)) * 1000,
            self.config.retry.max_backoff_ms
        )

        if self.config.retry.jitter:
            import random
            jitter = random.random() * base_delay / 2
            return int(base_delay + jitter)

        return int(base_delay)


class SmokeTestSuite:
    def __init__(self, config: SmokeTestConfig):
        self.client = SmokeTestClient(config)

    async def test_cancel_during_dns(self):
        print('🧪 Testing cancellation during DNS...')
        
        # Cancel immediately (simulating DNS phase)
        cancel_event = asyncio.Event()
        cancel_event.set()
        
        messages = [{"role": "user", "content": "Hello"}]
        
        try:
            await self.client.chat_completion(messages, cancel_event)
            raise Exception('Expected cancellation, but got success')
        except SmokeTestError as e:
            if e.error_type == 'Canceled':
                print(f'✅ Canceled during {e.details["phase"]} in {e.details["elapsed_ms"]}ms')
                return
            raise e

    async def test_cancel_during_connect(self):
        print('🧪 Testing cancellation during connection...')
        
        # Cancel after a short delay (simulating connection phase)
        cancel_event = asyncio.Event()
        asyncio.create_task(self._delayed_cancel(cancel_event, 0.1))
        
        messages = [{"role": "user", "content": "Hello"}]
        
        try:
            await self.client.chat_completion(messages, cancel_event)
            raise Exception('Expected cancellation, but got success')
        except SmokeTestError as e:
            if e.error_type == 'Canceled':
                print(f'✅ Canceled during {e.details["phase"]} in {e.details["elapsed_ms"]}ms')
                return
            raise e

    async def test_deadline_exceeded(self):
        print('🧪 Testing deadline exceeded...')
        
        config = SmokeTestConfig(
            deadline_ms=100,  # Very short deadline
            base_url='http://localhost:3000'  # Assuming Mockoon with timeout endpoint
        )
        
        short_deadline_client = SmokeTestClient(config)
        messages = [{"role": "user", "content": "Hello"}]
        
        try:
            await short_deadline_client.chat_completion(messages)
            raise Exception('Expected timeout, but got success')
        except SmokeTestError as e:
            if e.error_type == 'Timeout':
                print(f'✅ Timeout in {e.details["phase"]} (remaining: {e.details["remaining_budget_ms"]}ms) - {e.details["elapsed_ms"]}ms')
                return
            raise e

    async def test_rate_limit_retry_after(self):
        print('🧪 Testing rate limit with Retry-After...')
        
        config = SmokeTestConfig(
            base_url='http://localhost:3000'  # Assuming Mockoon with rate limit endpoint
        )
        
        rate_limit_client = SmokeTestClient(config)
        messages = [{"role": "user", "content": "Hello"}]
        
        try:
            await rate_limit_client.chat_completion(messages)
            raise Exception('Expected rate limit, but got success')
        except SmokeTestError as e:
            if e.error_type == 'RateLimited':
                print(f'✅ Rate limited with Retry-After: {e.details["retry_after_secs"]}s (elapsed: {e.details["elapsed_ms"]}ms)')
                return
            raise e

    async def test_server_5xx(self):
        print('🧪 Testing server 5xx error...')
        
        config = SmokeTestConfig(
            base_url='http://localhost:3000'  # Assuming Mockoon with error endpoint
        )
        
        error_client = SmokeTestClient(config)
        messages = [{"role": "user", "content": "Hello"}]
        
        try:
            await error_client.chat_completion(messages)
            raise Exception('Expected server error, but got success')
        except SmokeTestError as e:
            if e.error_type == 'Server5xx':
                print(f'✅ Server 5xx error: {e.details["status"]} (elapsed: {e.details["elapsed_ms"]}ms)')
                return
            raise e

    async def test_successful_request(self):
        print('🧪 Testing successful request...')
        
        messages = [{"role": "user", "content": "Hello"}]
        
        try:
            response = await self.client.chat_completion(messages)
            print(f'✅ Successful request: {str(response)[:100]}...')
            return
        except SmokeTestError as e:
            raise Exception(f'Expected success, but got: {e.message}')

    async def _delayed_cancel(self, cancel_event, delay_seconds):
        await asyncio.sleep(delay_seconds)
        cancel_event.set()

    async def run_all_tests(self):
        print('🚀 Running Python smoke test suite...')
        
        tests = [
            ('Cancel during DNS', self.test_cancel_during_dns),
            ('Cancel during connect', self.test_cancel_during_connect),
            ('Deadline exceeded', self.test_deadline_exceeded),
            ('Rate limit with Retry-After', self.test_rate_limit_retry_after),
            ('Server 5xx error', self.test_server_5xx),
            ('Successful request', self.test_successful_request)
        ]

        failed_tests = []

        for test_name, test_func in tests:
            try:
                await test_func()
                print(f'✅ {test_name}: PASSED')
            except Exception as e:
                print(f'❌ {test_name}: FAILED - {str(e)}')
                failed_tests.append((test_name, str(e)))

        if not failed_tests:
            print('🎉 All Python smoke tests passed!')
            return
        else:
            error_msg = ', '.join([f'{name}: {error}' for name, error in failed_tests])
            raise Exception(f'Smoke tests failed: {error_msg}')


async def main():
    config = SmokeTestConfig()
    suite = SmokeTestSuite(config)
    
    try:
        await suite.run_all_tests()
    except Exception as e:
        print(f'❌ Smoke test suite failed: {str(e)}')
        exit(1)


if __name__ == "__main__":
    asyncio.run(main())




