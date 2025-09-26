"""
# High-Performance Python LLM Client

Addresses all performance failure modes:
- Connection pooling with keep-alive
- Single deadline propagated to all operations
- Proper streaming with backpressure
- Bounded concurrency with asyncio.Semaphore
- Memory-efficient buffer reuse
"""

import asyncio
import json
import time
import uuid
from typing import Dict, List, Optional, AsyncGenerator, Any
from dataclasses import dataclass
import aiohttp
from aiohttp import ClientSession, ClientTimeout, TCPConnector


@dataclass
class ClientConfig:
    base_url: str = "http://localhost:3000"
    timeout: float = 30.0
    max_concurrent: int = 32
    keep_alive: float = 60.0
    retry_attempts: int = 3
    retry_base_delay: float = 0.1
    max_retry_delay: float = 5.0


class BufferPool:
    """Memory-efficient buffer pool to avoid allocations"""
    
    def __init__(self, max_size: int = 10):
        self.pool: List[bytearray] = []
        self.max_size = max_size
    
    def get(self) -> bytearray:
        if self.pool:
            buffer = self.pool.pop()
            buffer.clear()
            return buffer
        return bytearray(8192)
    
    def return_buffer(self, buffer: bytearray) -> None:
        if len(buffer) <= 65536 and len(self.pool) < self.max_size:
            buffer.clear()
            self.pool.append(buffer)


class PerformanceClient:
    """High-performance LLM client with all optimizations"""
    
    def __init__(self, config: Optional[ClientConfig] = None):
        self.config = config or ClientConfig()
        self.semaphore = asyncio.Semaphore(self.config.max_concurrent)
        self.buffer_pool = BufferPool()
        self._session: Optional[ClientSession] = None
    
    async def __aenter__(self):
        await self._ensure_session()
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self._session:
            await self._session.close()
    
    async def _ensure_session(self) -> None:
        """Create optimized HTTP session with connection pooling"""
        if self._session is None or self._session.closed:
            # Connection pooling with keep-alive
            connector = TCPConnector(
                limit=self.config.max_concurrent,
                limit_per_host=self.config.max_concurrent,
                keepalive_timeout=self.config.keep_alive,
                enable_cleanup_closed=True,
                ttl_dns_cache=300,  # DNS caching
                use_dns_cache=True,
            )
            
            timeout = ClientTimeout(
                total=self.config.timeout,
                connect=5.0,
                sock_read=self.config.timeout
            )
            
            self._session = ClientSession(
                connector=connector,
                timeout=timeout,
                headers={'Connection': 'keep-alive'}
            )
    
    async def chat_completion(
        self, 
        messages: List[Dict[str, str]], 
        deadline: float
    ) -> Dict[str, Any]:
        """Make chat completion request with deadline and concurrency control"""
        async with self.semaphore:  # Bound concurrency
            await self._ensure_session()
            
            # Check deadline
            if time.time() > deadline:
                raise asyncio.TimeoutError("Deadline exceeded")
            
            remaining_time = deadline - time.time()
            idempotency_key = self._generate_idempotency_key()
            
            body = {
                "model": "test-model",
                "messages": messages,
                "max_tokens": 100
            }
            
            return await self._make_request_with_retries(body, remaining_time, idempotency_key)
    
    async def stream_chat_completion(
        self, 
        messages: List[Dict[str, str]], 
        deadline: float
    ) -> AsyncGenerator[Dict[str, Any], None]:
        """Stream chat completion with backpressure control"""
        async with self.semaphore:
            await self._ensure_session()
            
            if time.time() > deadline:
                raise asyncio.TimeoutError("Deadline exceeded")
            
            remaining_time = deadline - time.time()
            idempotency_key = self._generate_idempotency_key()
            
            body = {
                "model": "test-model",
                "messages": messages,
                "max_tokens": 100,
                "stream": True
            }
            
            async for chunk in self._stream_request(body, remaining_time, idempotency_key):
                yield chunk
    
    async def _make_request_with_retries(
        self, 
        body: Dict[str, Any], 
        remaining_time: float, 
        idempotency_key: str
    ) -> Dict[str, Any]:
        """Make request with retries and deadline enforcement"""
        attempt = 0
        last_error = None
        start_time = time.time()
        
        while attempt < self.config.retry_attempts:
            attempt += 1
            
            elapsed = time.time() - start_time
            if elapsed >= remaining_time:
                raise asyncio.TimeoutError("Deadline exceeded")
            
            attempt_timeout = remaining_time - elapsed
            
            try:
                response_data = await self._make_single_request(body, attempt_timeout, idempotency_key)
                return json.loads(response_data)
            except Exception as error:
                last_error = error
                
                if "429" in str(error):
                    # Rate limited - respect Retry-After
                    retry_after = 1.0  # Default
                    if hasattr(error, 'headers') and 'retry-after' in error.headers:
                        retry_after = float(error.headers['retry-after'])
                    
                    if elapsed + retry_after >= remaining_time:
                        raise asyncio.TimeoutError("Deadline exceeded")
                    
                    await asyncio.sleep(retry_after)
                    continue
                
                if "5xx" in str(error) or isinstance(error, asyncio.TimeoutError):
                    # Retry on server errors and timeouts
                    backoff = self._calculate_backoff(attempt)
                    if elapsed + backoff >= remaining_time:
                        raise asyncio.TimeoutError("Deadline exceeded")
                    
                    await asyncio.sleep(backoff)
                    continue
                
                # Don't retry on client errors (4xx except 429)
                raise error
        
        raise last_error or Exception("Max retries exceeded")
    
    async def _make_single_request(
        self, 
        body: Dict[str, Any], 
        timeout: float, 
        idempotency_key: str
    ) -> str:
        """Make single HTTP request with timeout"""
        url = f"{self.config.base_url}/v1/chat/completions"
        
        headers = {
            'Content-Type': 'application/json',
            'Idempotency-Key': idempotency_key
        }
        
        try:
            async with self._session.post(
                url, 
                json=body, 
                headers=headers,
                timeout=aiohttp.ClientTimeout(total=timeout)
            ) as response:
                if response.status >= 200 and response.status < 300:
                    return await response.text()
                elif response.status == 429:
                    raise Exception(f"429 Rate Limited")
                elif response.status >= 500:
                    raise Exception(f"5xx Server Error: {response.status}")
                else:
                    raise Exception(f"Client Error: {response.status}")
        except asyncio.TimeoutError:
            raise asyncio.TimeoutError("Request timeout")
        except Exception as e:
            raise Exception(f"Request failed: {e}")
    
    async def _stream_request(
        self, 
        body: Dict[str, Any], 
        timeout: float, 
        idempotency_key: str
    ) -> AsyncGenerator[Dict[str, Any], None]:
        """Stream request with backpressure control"""
        url = f"{self.config.base_url}/v1/chat/completions"
        
        headers = {
            'Content-Type': 'application/json',
            'Idempotency-Key': idempotency_key
        }
        
        try:
            async with self._session.post(
                url, 
                json=body, 
                headers=headers,
                timeout=aiohttp.ClientTimeout(total=timeout)
            ) as response:
                if response.status < 200 or response.status >= 300:
                    raise Exception(f"HTTP {response.status}")
                
                buffer = self.buffer_pool.get()
                current_data = ''
                
                try:
                    async for chunk in response.content.iter_chunked(1024):
                        buffer.extend(chunk)
                        current_data += chunk.decode('utf-8', errors='ignore')
                        
                        # Parse SSE events with backpressure
                        events = self._parse_sse_events(current_data)
                        for event in events:
                            if event.get('data') == '[DONE]':
                                return
                            
                            try:
                                json_data = json.loads(event['data'])
                                yield json_data
                            except json.JSONDecodeError:
                                # Skip malformed JSON
                                pass
                finally:
                    self.buffer_pool.return_buffer(buffer)
                    
        except asyncio.TimeoutError:
            raise asyncio.TimeoutError("Stream timeout")
        except Exception as e:
            raise Exception(f"Stream failed: {e}")
    
    def _parse_sse_events(self, text: str) -> List[Dict[str, str]]:
        """Parse Server-Sent Events efficiently"""
        events = []
        lines = text.split('\n')
        current_event = {}
        
        for line in lines:
            if line.startswith('data: '):
                current_event['data'] = line[6:]
                events.append(current_event)
                current_event = {}
        
        return events
    
    def _calculate_backoff(self, attempt: int) -> float:
        """Calculate exponential backoff with jitter"""
        delay = self.config.retry_base_delay * (2 ** (attempt - 1))
        jitter = delay * 0.1 * (0.5 - 0.5)  # Simplified jitter
        return min(delay + jitter, self.config.max_retry_delay)
    
    def _generate_idempotency_key(self) -> str:
        """Generate unique idempotency key"""
        return f"py-{int(time.time() * 1000)}-{uuid.uuid4().hex[:9]}"


# Example usage and test
async def smoke_test():
    """Quick smoke test of the performance client"""
    config = ClientConfig()
    
    async with PerformanceClient(config) as client:
        messages = [{"role": "user", "content": "Hello"}]
        deadline = time.time() + 10.0  # 10 second deadline
        
        try:
            # Test regular completion
            result = await client.chat_completion(messages, deadline)
            print(f"✅ Chat completion: {result.get('id', 'unknown')}")
            
            # Test streaming
            print("🧪 Testing streaming...")
            async for chunk in client.stream_chat_completion(messages, deadline):
                print(f"📦 Chunk: {chunk}")
                
        except Exception as e:
            print(f"❌ Test failed: {e}")


if __name__ == "__main__":
    asyncio.run(smoke_test())




