#!/usr/bin/env python3
"""
Async functionality tests for LightLLM Rust Python bindings.

Tests async/await support, concurrent operations, and asyncio integration
to ensure the async bindings work correctly and efficiently.
"""

import pytest
import asyncio
import time
import gc
import weakref
from typing import Dict, Any, List, Optional
import logging

# Configure logging for tests
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Import the bindings
try:
    import nexus_nitro_llm
    from nexus_nitro_llm import (
        PyConfig, PyMessage, PyAsyncLightLLMClient, PyAsyncStreamingClient,
        LightLLMError, ConnectionError, ConfigurationError
    )
    BINDINGS_AVAILABLE = True
except ImportError:
    BINDINGS_AVAILABLE = False
    nexus_nitro_llm = None


class TestAsyncFunctionality:
    """Test async functionality of Python bindings."""

    @pytest.fixture(autouse=True)
    def setup_method(self):
        """Set up test environment before each test."""
        if not BINDINGS_AVAILABLE:
            pytest.skip("Python bindings not available - run 'maturin develop --features python' first")

    @pytest.mark.asyncio
    async def test_async_client_creation(self):
        """Test async client creation and basic properties."""
        config = PyConfig(
            lightllm_url="http://localhost:8000",
            model_id="test-model",
            token="test-token",
            timeout=30
        )

        # Test async client creation
        async_client = PyAsyncLightLLMClient(config)
        assert async_client is not None

        # Test stats retrieval
        stats = async_client.get_stats()
        assert isinstance(stats, dict)
        assert "adapter_type" in stats
        assert "async_enabled" in stats
        assert stats["async_enabled"] is True
        assert stats["runtime_type"] == "async"

    @pytest.mark.asyncio
    async def test_async_chat_completions(self):
        """Test async chat completions functionality."""
        config = PyConfig(
            lightllm_url="http://localhost:8000",
            model_id="test-model"
        )

        async_client = PyAsyncLightLLMClient(config)
        
        # Create test messages
        messages = [
            nexus_nitro_llm.create_message("user", "Hello, this is a test message.")
        ]

        # Test async chat completions
        response = await async_client.chat_completions_async(
            messages=messages,
            max_tokens=50,
            temperature=0.7
        )

        assert isinstance(response, dict)
        assert "id" in response
        assert "object" in response
        assert "choices" in response
        assert "usage" in response
        assert response["object"] == "chat.completion"

    @pytest.mark.asyncio
    async def test_async_concurrent_requests(self):
        """Test concurrent async requests."""
        config = PyConfig(
            lightllm_url="http://localhost:8000",
            model_id="test-model"
        )

        async_client = PyAsyncLightLLMClient(config)
        
        # Create multiple test messages
        messages_list = [
            [nexus_nitro_llm.create_message("user", f"Test message {i}")]
            for i in range(5)
        ]

        # Test concurrent requests
        start_time = time.time()
        tasks = [
            async_client.chat_completions_async(
                messages=messages,
                max_tokens=20,
                temperature=0.5
            )
            for messages in messages_list
        ]
        
        responses = await asyncio.gather(*tasks, return_exceptions=True)
        elapsed = time.time() - start_time

        # Verify all requests completed
        assert len(responses) == 5
        successful_responses = [r for r in responses if not isinstance(r, Exception)]
        assert len(successful_responses) == 5

        # Verify response format
        for response in successful_responses:
            assert isinstance(response, dict)
            assert "choices" in response

        # Verify concurrent execution (should be faster than sequential)
        assert elapsed < 5.0  # Should complete in under 5 seconds

    @pytest.mark.asyncio
    async def test_async_error_handling(self):
        """Test async error handling."""
        config = PyConfig(
            lightllm_url="http://localhost:8000",
            model_id="test-model"
        )

        async_client = PyAsyncLightLLMClient(config)

        # Test empty messages error
        with pytest.raises(LightLLMError):
            await async_client.chat_completions_async(
                messages=[],
                max_tokens=50
            )

        # Test invalid temperature error
        with pytest.raises(LightLLMError):
            await async_client.chat_completions_async(
                messages=[nexus_nitro_llm.create_message("user", "test")],
                temperature=3.0  # Invalid temperature
            )

    @pytest.mark.asyncio
    async def test_async_connection_testing(self):
        """Test async connection testing."""
        config = PyConfig(
            lightllm_url="http://localhost:8000",
            model_id="test-model"
        )

        async_client = PyAsyncLightLLMClient(config)

        # Test async connection
        is_connected = await async_client.test_connection_async()
        assert isinstance(is_connected, bool)

    @pytest.mark.asyncio
    async def test_async_streaming_client(self):
        """Test async streaming client."""
        config = PyConfig(
            lightllm_url="http://localhost:8000",
            model_id="test-model"
        )

        async_streaming_client = PyAsyncStreamingClient(config)
        assert async_streaming_client is not None

        # Test async streaming
        messages = [nexus_nitro_llm.create_message("user", "Test streaming message")]
        
        response = await async_streaming_client.stream_chat_completions_async(
            messages=messages,
            max_tokens=50
        )

        assert isinstance(response, dict)
        assert "choices" in response

    @pytest.mark.asyncio
    async def test_async_performance_metrics(self):
        """Test async performance metrics collection."""
        config = PyConfig(
            lightllm_url="http://localhost:8000",
            model_id="test-model"
        )

        async_client = PyAsyncLightLLMClient(config)
        
        # Get initial stats
        initial_stats = async_client.get_stats()
        initial_requests = initial_stats["total_requests"]
        initial_errors = initial_stats["total_errors"]

        # Make some requests
        messages = [nexus_nitro_llm.create_message("user", "Performance test")]
        
        for i in range(3):
            await async_client.chat_completions_async(
                messages=messages,
                max_tokens=10
            )

        # Get updated stats
        updated_stats = async_client.get_stats()
        updated_requests = updated_stats["total_requests"]
        updated_errors = updated_stats["total_errors"]

        # Verify metrics were updated
        assert updated_requests == initial_requests + 3
        assert updated_errors == initial_errors  # No errors expected

    @pytest.mark.asyncio
    async def test_async_memory_management(self):
        """Test async memory management and cleanup."""
        refs = []

        # Create async clients and keep weak references
        for i in range(10):
            config = PyConfig(
                lightllm_url=f"http://localhost:800{i % 10}",
                model_id=f"model-{i}"
            )
            async_client = PyAsyncLightLLMClient(config)
            refs.append(weakref.ref(async_client))

        # Force garbage collection
        gc.collect()

        # All objects should be cleaned up
        live_refs = sum(1 for ref in refs if ref() is not None)
        assert live_refs == 0, f"Memory leak detected: {live_refs} async objects still alive"

    @pytest.mark.asyncio
    async def test_async_concurrent_config_access(self):
        """Test concurrent access to async configurations."""
        async def create_async_configs(count: int) -> List[PyAsyncLightLLMClient]:
            configs = []
            for i in range(count):
                config = PyConfig(
                    lightllm_url=f"http://localhost:{8000 + i}",
                    model_id=f"model-{i}"
                )
                async_client = PyAsyncLightLLMClient(config)
                configs.append(async_client)
            return configs

        # Create configurations concurrently
        tasks = [create_async_configs(10) for _ in range(5)]
        results = await asyncio.gather(*tasks)

        # Flatten results
        all_configs = [config for configs in results for config in configs]
        assert len(all_configs) == 50

        # Verify configurations are correct
        for i, async_client in enumerate(all_configs):
            stats = async_client.get_stats()
            assert isinstance(stats, dict)
            assert "async_enabled" in stats
            assert stats["async_enabled"] is True

    @pytest.mark.asyncio
    async def test_async_semaphore_limiting(self):
        """Test async request limiting with semaphores."""
        config = PyConfig(
            lightllm_url="http://localhost:8000",
            model_id="test-model"
        )

        async_client = PyAsyncLightLLMClient(config)
        
        # Create semaphore to limit concurrent requests
        semaphore = asyncio.Semaphore(2)
        request_count = 0
        max_concurrent = 0
        current_concurrent = 0

        async def limited_request(request_id: int):
            nonlocal request_count, max_concurrent, current_concurrent
            
            async with semaphore:
                current_concurrent += 1
                max_concurrent = max(max_concurrent, current_concurrent)
                
                messages = [nexus_nitro_llm.create_message("user", f"Request {request_id}")]
                response = await async_client.chat_completions_async(
                    messages=messages,
                    max_tokens=10
                )
                
                current_concurrent -= 1
                request_count += 1
                return response

        # Create 5 requests with semaphore limit of 2
        tasks = [limited_request(i) for i in range(5)]
        responses = await asyncio.gather(*tasks)

        # Verify all requests completed
        assert len(responses) == 5
        assert request_count == 5
        assert max_concurrent <= 2  # Semaphore should have limited concurrency

    @pytest.mark.asyncio
    async def test_async_timeout_handling(self):
        """Test async timeout handling."""
        config = PyConfig(
            lightllm_url="http://localhost:8000",
            model_id="test-model",
            timeout=1  # Very short timeout
        )

        async_client = PyAsyncLightLLMClient(config)
        
        messages = [nexus_nitro_llm.create_message("user", "Timeout test message")]

        # This should either complete quickly or timeout
        try:
            response = await asyncio.wait_for(
                async_client.chat_completions_async(
                    messages=messages,
                    max_tokens=10
                ),
                timeout=2.0  # Python-level timeout
            )
            assert isinstance(response, dict)
        except asyncio.TimeoutError:
            # Timeout is acceptable for this test
            pass
        except Exception as e:
            # Other exceptions are also acceptable (connection errors, etc.)
            assert isinstance(e, (ConnectionError, LightLLMError))


class TestAsyncPerformance:
    """Test async performance characteristics."""

    @pytest.fixture(autouse=True)
    def setup_method(self):
        """Set up test environment before each test."""
        if not BINDINGS_AVAILABLE:
            pytest.skip("Python bindings not available")

    @pytest.mark.asyncio
    async def test_async_throughput(self):
        """Test async throughput performance."""
        config = PyConfig(
            lightllm_url="http://localhost:8000",
            model_id="test-model"
        )

        async_client = PyAsyncLightLLMClient(config)
        
        # Test throughput with multiple concurrent requests
        num_requests = 10
        messages_list = [
            [nexus_nitro_llm.create_message("user", f"Throughput test {i}")]
            for i in range(num_requests)
        ]

        start_time = time.time()
        tasks = [
            async_client.chat_completions_async(
                messages=messages,
                max_tokens=5
            )
            for messages in messages_list
        ]
        
        responses = await asyncio.gather(*tasks)
        elapsed = time.time() - start_time

        # Calculate throughput
        throughput = num_requests / elapsed
        
        # Should handle at least 5 requests per second
        assert throughput >= 5.0, f"Async throughput too low: {throughput:.2f} req/sec"
        assert len(responses) == num_requests

    @pytest.mark.asyncio
    async def test_async_latency_consistency(self):
        """Test async latency consistency."""
        config = PyConfig(
            lightllm_url="http://localhost:8000",
            model_id="test-model"
        )

        async_client = PyAsyncLightLLMClient(config)
        
        messages = [nexus_nitro_llm.create_message("user", "Latency test")]
        
        # Measure latency for multiple requests
        latencies = []
        for i in range(5):
            start_time = time.time()
            await async_client.chat_completions_async(
                messages=messages,
                max_tokens=5
            )
            latency = time.time() - start_time
            latencies.append(latency)

        # Calculate statistics
        avg_latency = sum(latencies) / len(latencies)
        max_latency = max(latencies)
        min_latency = min(latencies)

        # Latency should be consistent (not too much variation)
        assert max_latency < avg_latency * 3, "Async latency too inconsistent"
        assert min_latency > 0, "Async latency should be positive"
        assert avg_latency < 2.0, f"Async latency too high: {avg_latency:.3f}s"


if __name__ == "__main__":
    # Run async tests when executed directly
    pytest.main([__file__, "-v", "-s"])
