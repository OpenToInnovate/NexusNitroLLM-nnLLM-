#!/usr/bin/env python3
"""
Async usage example for LightLLM Rust Python bindings.

This example demonstrates async/await support for Python applications that use asyncio.
Perfect for web servers, async frameworks, and concurrent applications.

Key async features:
- Non-blocking async/await support
- Integration with Python's asyncio event loop
- Concurrent request handling
- Async streaming support
- Performance monitoring
"""

import asyncio
import time
import logging
from typing import List, Dict, Any, Optional
from concurrent.futures import ThreadPoolExecutor

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

try:
    import nexus_nitro_llm
    from nexus_nitro_llm import (
        PyConfig, PyMessage, PyAsyncLightLLMClient, PyAsyncStreamingClient,
        LightLLMError, ConnectionError, ConfigurationError
    )
    BINDINGS_AVAILABLE = True
except ImportError as e:
    logger.error(f"Failed to import nexus_nitro_llm: {e}")
    logger.error("Please install the Python bindings: pip install -e .")
    BINDINGS_AVAILABLE = False
    nexus_nitro_llm = None


class AsyncLLMProcessor:
    """Async LLM processor using Rust bindings with asyncio support."""

    def __init__(self, backend_url: str, model_id: str = "llama", token: Optional[str] = None):
        """Initialize with async-compatible configuration."""
        
        if not BINDINGS_AVAILABLE:
            raise ImportError("LightLLM Rust bindings not available. Please install with: pip install -e .")

        try:
            # Create configuration with validation
            self.config = PyConfig(
                lightllm_url=backend_url,
                model_id=model_id,
                token=token,
                timeout=30  # 30 second timeout
            )

            # Enable all performance optimizations
            self.config.set_connection_pooling(True)

            # Create async clients
            self.async_client = PyAsyncLightLLMClient(self.config)
            self.async_streaming_client = PyAsyncStreamingClient(self.config)

            logger.info(f"🚀 Async LLM processor initialized")
            logger.info(f"   Backend: {backend_url}")
            logger.info(f"   Model: {model_id}")
            logger.info(f"   Async support: Enabled")
            logger.info(f"   Token: {'Set' if token else 'Not set'}")

        except ConfigurationError as e:
            logger.error(f"Configuration error: {e}")
            raise
        except Exception as e:
            logger.error(f"Failed to initialize async processor: {e}")
            raise

    async def single_async_request(
        self,
        prompt: str,
        max_tokens: int = 100,
        temperature: float = 0.7
    ) -> Dict[str, Any]:
        """Send a single async request."""
        
        messages = [nexus_nitro_llm.create_message("user", prompt)]
        
        start_time = time.time()
        response = await self.async_client.chat_completions_async(
            messages=messages,
            max_tokens=max_tokens,
            temperature=temperature
        )
        elapsed = (time.time() - start_time) * 1000
        
        logger.info(f"⚡ Async request completed in {elapsed:.1f}ms")
        return response

    async def concurrent_requests(
        self,
        prompts: List[str],
        max_tokens: int = 100,
        max_concurrent: int = 10
    ) -> List[Dict[str, Any]]:
        """Process multiple requests concurrently with asyncio."""
        
        logger.info(f"🔄 Processing {len(prompts)} requests concurrently...")
        start_time = time.time()
        
        # Create semaphore to limit concurrent requests
        semaphore = asyncio.Semaphore(max_concurrent)
        
        async def process_single(prompt: str) -> Dict[str, Any]:
            async with semaphore:
                messages = [nexus_nitro_llm.create_message("user", prompt)]
                return await self.async_client.chat_completions_async(
                    messages=messages,
                    max_tokens=max_tokens,
                    temperature=0.7
                )
        
        # Process all requests concurrently
        tasks = [process_single(prompt) for prompt in prompts]
        responses = await asyncio.gather(*tasks, return_exceptions=True)
        
        elapsed = (time.time() - start_time) * 1000
        successful_responses = [r for r in responses if not isinstance(r, Exception)]
        
        logger.info(f"✅ Concurrent processing completed: {len(successful_responses)} responses in {elapsed:.1f}ms")
        logger.info(f"   Average per request: {elapsed/len(prompts):.1f}ms")
        logger.info(f"   Throughput: {len(prompts)/(elapsed/1000):.1f} requests/second")
        
        return successful_responses

    async def async_streaming_request(
        self,
        prompt: str,
        max_tokens: int = 200
    ) -> Dict[str, Any]:
        """Send an async streaming request."""
        
        messages = [nexus_nitro_llm.create_message("user", prompt)]
        
        logger.info("🌊 Starting async streaming request...")
        start_time = time.time()
        
        response = await self.async_streaming_client.stream_chat_completions_async(
            messages=messages,
            max_tokens=max_tokens,
            temperature=0.7
        )
        
        elapsed = (time.time() - start_time) * 1000
        logger.info(f"🌊 Async streaming response received in {elapsed:.1f}ms")
        
        return response

    async def async_conversation_demo(self):
        """Demonstrate async multi-turn conversation."""
        
        logger.info("\n💬 Async multi-turn conversation demo")
        logger.info("-" * 50)
        
        conversation_history = []
        
        turns = [
            "What is machine learning?",
            "Can you give me a practical example?",
            "How does this relate to neural networks?",
            "What are the main challenges?"
        ]
        
        total_time = 0
        
        for i, user_input in enumerate(turns, 1):
            logger.info(f"\n👤 User: {user_input}")
            
            # Build conversation history efficiently
            messages = [nexus_nitro_llm.create_message("system", "You are a knowledgeable AI assistant.")]
            
            # Add conversation history
            for role, content in conversation_history:
                messages.append(nexus_nitro_llm.create_message(role, content))
            
            # Add current user message
            messages.append(nexus_nitro_llm.create_message("user", user_input))
            
            # Send async request
            start_time = time.time()
            response = await self.async_client.chat_completions_async(
                messages=messages,
                max_tokens=150,
                temperature=0.7
            )
            request_time = (time.time() - start_time) * 1000
            total_time += request_time
            
            # Extract assistant response
            if isinstance(response, dict) and 'choices' in response:
                assistant_response = response['choices'][0]['message']['content']
                logger.info(f"🤖 Assistant: {assistant_response}")
                
                # Add to history for next turn
                conversation_history.append(("user", user_input))
                conversation_history.append(("assistant", assistant_response))
            else:
                logger.info(f"🤖 Assistant: [Response format: {type(response)}]")
            
            logger.info(f"   ⚡ Async response time: {request_time:.1f}ms")
        
        logger.info(f"\n📊 Async conversation statistics:")
        logger.info(f"   Total turns: {len(turns)}")
        logger.info(f"   Total time: {total_time:.1f}ms")
        logger.info(f"   Average per turn: {total_time/len(turns):.1f}ms")
        logger.info(f"   Async efficiency: Non-blocking event loop integration")

    async def performance_comparison(self):
        """Compare async vs sync performance."""
        
        logger.info("\n📈 Async vs Sync Performance Comparison")
        logger.info("=" * 50)
        
        prompts = [
            "What is artificial intelligence?",
            "Explain machine learning briefly.",
            "What are neural networks?",
            "Define deep learning.",
            "What is natural language processing?",
        ]
        
        # Test async performance
        logger.info("Testing async performance...")
        start_time = time.time()
        async_responses = await self.concurrent_requests(prompts, max_concurrent=5)
        async_time = (time.time() - start_time) * 1000
        
        logger.info(f"Async results: {len(async_responses)} responses in {async_time:.1f}ms")
        logger.info(f"Async throughput: {len(prompts)/(async_time/1000):.1f} requests/second")
        
        # Get performance stats
        stats = self.async_client.get_stats()
        logger.info(f"Async client stats: {stats}")


async def main():
    """Main async demonstration."""
    
    logger.info("🚀 LightLLM Rust Python Bindings - Async Usage")
    logger.info("=" * 60)
    
    # Initialize async processor
    try:
        processor = AsyncLLMProcessor(
            backend_url="http://localhost:8000",
            model_id="llama"
        )
    except Exception as e:
        logger.error(f"❌ Failed to initialize async processor: {e}")
        logger.error("💡 Make sure LightLLM backend is running on localhost:8000")
        return
    
    # Test single async request
    logger.info("\n1. 🎯 Single Async Request Test")
    logger.info("-" * 40)
    try:
        response = await processor.single_async_request(
            "Explain the benefits of async programming in Python in one sentence.",
            max_tokens=50
        )
        logger.info("✅ Single async request successful")
    except Exception as e:
        logger.error(f"❌ Single async request failed: {e}")
    
    # Test concurrent async requests
    logger.info("\n2. 🔄 Concurrent Async Requests Test")
    logger.info("-" * 40)
    prompts = [
        "What is artificial intelligence?",
        "Explain machine learning briefly.",
        "What are neural networks?",
        "Define deep learning.",
        "What is natural language processing?",
    ]
    
    try:
        concurrent_responses = await processor.concurrent_requests(prompts, max_tokens=30, max_concurrent=3)
        logger.info(f"✅ Concurrent async processing successful: {len(concurrent_responses)} responses")
    except Exception as e:
        logger.error(f"❌ Concurrent async processing failed: {e}")
    
    # Test async streaming
    logger.info("\n3. 🌊 Async Streaming Test")
    logger.info("-" * 40)
    try:
        stream_response = await processor.async_streaming_request(
            "Write a short poem about the speed of async programming.",
            max_tokens=100
        )
        logger.info("✅ Async streaming request successful")
    except Exception as e:
        logger.error(f"❌ Async streaming request failed: {e}")
    
    # Async conversation demo
    await processor.async_conversation_demo()
    
    # Performance comparison
    await processor.performance_comparison()
    
    logger.info("\n✨ Async usage demonstration complete!")
    logger.info("\n🎯 Async Performance Highlights:")
    logger.info("  • Non-blocking async/await support")
    logger.info("  • Integration with Python's asyncio event loop")
    logger.info("  • Concurrent request processing")
    logger.info("  • Zero-copy data transfer optimizations")
    logger.info("  • Memory-safe operations with Rust guarantees")
    logger.info("  • Async runtime efficiency")


async def benchmark_async_scaling():
    """Benchmark async scaling characteristics."""
    
    logger.info("\n📈 Async Scaling Benchmark")
    logger.info("=" * 30)
    
    processor = AsyncLLMProcessor("http://localhost:8000")
    
    # Test different concurrency levels
    concurrency_levels = [1, 5, 10, 20]
    
    for concurrency in concurrency_levels:
        logger.info(f"\nTesting concurrency level: {concurrency}")
        prompts = [f"Test prompt {i}" for i in range(concurrency * 2)]  # 2x requests as concurrency
        
        try:
            start_time = time.time()
            responses = await processor.concurrent_requests(
                prompts, 
                max_tokens=10, 
                max_concurrent=concurrency
            )
            total_time = (time.time() - start_time) * 1000
            
            logger.info(f"  Total time: {total_time:.1f}ms")
            logger.info(f"  Per request: {total_time/len(prompts):.1f}ms")
            logger.info(f"  Throughput: {len(prompts)/(total_time/1000):.1f} req/sec")
            logger.info(f"  Concurrency efficiency: {len(responses)/len(prompts)*100:.1f}%")
            
        except Exception as e:
            logger.error(f"  Failed: {e}")


if __name__ == "__main__":
    # Run the main async demonstration
    asyncio.run(main())
    
    # Uncomment to run scaling benchmark
    # asyncio.run(benchmark_async_scaling())
