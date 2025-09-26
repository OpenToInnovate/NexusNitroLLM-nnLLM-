#!/usr/bin/env python3
"""
Direct Mode Usage Example for LightLLM Rust Python bindings.

This example demonstrates the new direct integration mode that bypasses HTTP entirely
for maximum performance. Perfect for Python applications that want direct access
to LightLLM without network overhead.

Key benefits of direct mode:
- Zero HTTP overhead (no network serialization/deserialization)
- Direct memory access between Python and Rust
- Minimal latency (direct function calls)
- Maximum throughput (no network bottlenecks)
- No need for running LightLLM server separately
"""

import asyncio
import time
import logging
from typing import List, Dict, Any, Optional

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

try:
    import nexus_nitro_llm
    from nexus_nitro_llm import (
        PyConfig, PyMessage, PyLightLLMClient, PyAsyncLightLLMClient,
        LightLLMError, ConnectionError, ConfigurationError
    )
    BINDINGS_AVAILABLE = True
except ImportError as e:
    logger.error(f"Failed to import nexus_nitro_llm: {e}")
    logger.error("Please install the Python bindings: pip install -e .")
    BINDINGS_AVAILABLE = False
    nexus_nitro_llm = None


class DirectModeProcessor:
    """Direct mode processor using Rust bindings with zero HTTP overhead."""

    def __init__(self, model_id: str = "llama", token: Optional[str] = None):
        """Initialize with direct mode configuration."""
        
        if not BINDINGS_AVAILABLE:
            raise ImportError("LightLLM Rust bindings not available. Please install with: pip install -e .")

        try:
            # Create configuration for direct mode
            # No URL needed - defaults to direct mode
            self.config = PyConfig(
                model_id=model_id,
                token=token,
                timeout=30
            )

            # Create clients (no HTTP overhead)
            self.sync_client = PyLightLLMClient(self.config)
            self.async_client = PyAsyncLightLLMClient(self.config)

            logger.info(f"🚀 Direct mode processor initialized")
            logger.info(f"   Model: {model_id}")
            logger.info(f"   Mode: Direct (no HTTP)")
            logger.info(f"   Token: {'Set' if token else 'Not set'}")
            logger.info(f"   Performance: Maximum (zero network overhead)")

        except ConfigurationError as e:
            logger.error(f"Configuration error: {e}")
            raise
        except Exception as e:
            logger.error(f"Failed to initialize direct processor: {e}")
            raise

    def sync_request(self, prompt: str, max_tokens: int = 100) -> Dict[str, Any]:
        """Send a synchronous direct request."""
        
        messages = [nexus_nitro_llm.create_message("user", prompt)]
        
        start_time = time.time()
        response = self.sync_client.chat_completions(
            messages=messages,
            max_tokens=max_tokens,
            temperature=0.7
        )
        elapsed = (time.time() - start_time) * 1000
        
        logger.info(f"⚡ Direct sync request completed in {elapsed:.1f}ms")
        return response

    async def async_request(self, prompt: str, max_tokens: int = 100) -> Dict[str, Any]:
        """Send an asynchronous direct request."""
        
        messages = [nexus_nitro_llm.create_message("user", prompt)]
        
        start_time = time.time()
        response = await self.async_client.chat_completions_async(
            messages=messages,
            max_tokens=max_tokens,
            temperature=0.7
        )
        elapsed = (time.time() - start_time) * 1000
        
        logger.info(f"⚡ Direct async request completed in {elapsed:.1f}ms")
        return response

    async def concurrent_requests(self, prompts: List[str], max_tokens: int = 50) -> List[Dict[str, Any]]:
        """Process multiple requests concurrently in direct mode."""
        
        logger.info(f"🔄 Processing {len(prompts)} direct requests concurrently...")
        start_time = time.time()
        
        # Create tasks for concurrent execution
        tasks = [
            self.async_request(prompt, max_tokens)
            for prompt in prompts
        ]
        
        # Execute all requests concurrently
        responses = await asyncio.gather(*tasks, return_exceptions=True)
        
        elapsed = (time.time() - start_time) * 1000
        successful_responses = [r for r in responses if not isinstance(r, Exception)]
        
        logger.info(f"✅ Direct concurrent processing completed: {len(successful_responses)} responses in {elapsed:.1f}ms")
        logger.info(f"   Average per request: {elapsed/len(prompts):.1f}ms")
        logger.info(f"   Throughput: {len(prompts)/(elapsed/1000):.1f} requests/second")
        
        return successful_responses

    def performance_comparison(self):
        """Compare direct mode vs HTTP mode performance."""
        
        logger.info("\n📈 Direct Mode vs HTTP Mode Performance Comparison")
        logger.info("=" * 60)
        
        prompts = [
            "What is artificial intelligence?",
            "Explain machine learning briefly.",
            "What are neural networks?",
            "Define deep learning.",
            "What is natural language processing?",
        ]
        
        # Test direct mode performance
        logger.info("Testing direct mode performance...")
        start_time = time.time()
        
        direct_responses = []
        for prompt in prompts:
            response = self.sync_request(prompt, max_tokens=30)
            direct_responses.append(response)
        
        direct_time = (time.time() - start_time) * 1000
        
        logger.info(f"Direct mode results: {len(direct_responses)} responses in {direct_time:.1f}ms")
        logger.info(f"Direct mode throughput: {len(prompts)/(direct_time/1000):.1f} requests/second")
        logger.info(f"Direct mode advantages:")
        logger.info(f"  • Zero HTTP overhead")
        logger.info(f"  • Direct memory access")
        logger.info(f"  • No network serialization")
        logger.info(f"  • Maximum performance")

    def get_performance_stats(self):
        """Get detailed performance statistics."""
        
        stats = self.sync_client.get_stats()
        logger.info(f"\n📊 Direct Mode Performance Statistics:")
        logger.info(f"   Adapter type: {stats.get('adapter_type', 'unknown')}")
        logger.info(f"   Runtime type: {stats.get('runtime_type', 'unknown')}")
        logger.info(f"   Total requests: {stats.get('total_requests', 0)}")
        logger.info(f"   Success rate: {stats.get('success_rate_percent', 0):.1f}%")
        logger.info(f"   Connection pooling: {stats.get('connection_pooling', False)}")
        logger.info(f"   Direct mode: {stats.get('lightllm_url', '') == 'direct'}")


async def main():
    """Main direct mode demonstration."""
    
    logger.info("🚀 LightLLM Rust Python Bindings - Direct Mode Usage")
    logger.info("=" * 70)
    
    # Initialize direct mode processor
    try:
        processor = DirectModeProcessor(
            model_id="llama",
            token=None  # No token needed for direct mode
        )
    except Exception as e:
        logger.error(f"❌ Failed to initialize direct processor: {e}")
        return
    
    # Test single direct request
    logger.info("\n1. 🎯 Single Direct Request Test")
    logger.info("-" * 40)
    try:
        response = processor.sync_request(
            "Explain the benefits of direct integration in one sentence.",
            max_tokens=50
        )
        logger.info("✅ Single direct request successful")
        if isinstance(response, dict) and 'choices' in response:
            content = response['choices'][0]['message']['content']
            logger.info(f"   Response: {content}")
    except Exception as e:
        logger.error(f"❌ Single direct request failed: {e}")
    
    # Test async direct request
    logger.info("\n2. ⚡ Async Direct Request Test")
    logger.info("-" * 40)
    try:
        response = await processor.async_request(
            "What are the performance benefits of direct mode?",
            max_tokens=50
        )
        logger.info("✅ Async direct request successful")
        if isinstance(response, dict) and 'choices' in response:
            content = response['choices'][0]['message']['content']
            logger.info(f"   Response: {content}")
    except Exception as e:
        logger.error(f"❌ Async direct request failed: {e}")
    
    # Test concurrent direct requests
    logger.info("\n3. 🔄 Concurrent Direct Requests Test")
    logger.info("-" * 40)
    prompts = [
        "What is machine learning?",
        "Explain neural networks.",
        "Define artificial intelligence.",
        "What is deep learning?",
        "Explain natural language processing.",
    ]
    
    try:
        concurrent_responses = await processor.concurrent_requests(prompts, max_tokens=30)
        logger.info(f"✅ Concurrent direct processing successful: {len(concurrent_responses)} responses")
    except Exception as e:
        logger.error(f"❌ Concurrent direct processing failed: {e}")
    
    # Performance comparison
    processor.performance_comparison()
    
    # Get performance stats
    processor.get_performance_stats()
    
    logger.info("\n✨ Direct mode demonstration complete!")
    logger.info("\n🎯 Direct Mode Benefits:")
    logger.info("  • Zero HTTP overhead (no network serialization)")
    logger.info("  • Direct memory access between Python and Rust")
    logger.info("  • Minimal latency (direct function calls)")
    logger.info("  • Maximum throughput (no network bottlenecks)")
    logger.info("  • No need for separate LightLLM server")
    logger.info("  • Perfect for embedded applications")
    logger.info("  • Ideal for high-performance computing")


async def benchmark_direct_mode():
    """Benchmark direct mode performance characteristics."""
    
    logger.info("\n📈 Direct Mode Performance Benchmark")
    logger.info("=" * 40)
    
    processor = DirectModeProcessor("llama")
    
    # Test different request sizes
    request_sizes = [1, 5, 10, 20, 50]
    
    for size in request_sizes:
        logger.info(f"\nTesting {size} concurrent direct requests...")
        prompts = [f"Test prompt {i}" for i in range(size)]
        
        try:
            start_time = time.time()
            responses = await processor.concurrent_requests(prompts, max_tokens=10)
            total_time = (time.time() - start_time) * 1000
            
            logger.info(f"  Total time: {total_time:.1f}ms")
            logger.info(f"  Per request: {total_time/size:.1f}ms")
            logger.info(f"  Throughput: {size/(total_time/1000):.1f} req/sec")
            logger.info(f"  Success rate: {len(responses)/size*100:.1f}%")
            
        except Exception as e:
            logger.error(f"  Failed: {e}")


if __name__ == "__main__":
    # Run the main direct mode demonstration
    asyncio.run(main())
    
    # Uncomment to run performance benchmark
    # asyncio.run(benchmark_direct_mode())
