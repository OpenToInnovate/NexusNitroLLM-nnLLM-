#!/usr/bin/env python3
"""
Enhanced advanced usage example for LightLLM Rust Python bindings.

This example demonstrates advanced features:
- High-performance streaming responses
- Batch processing multiple requests
- Connection pooling optimization
- Memory-efficient operations
- Comprehensive error handling
- Performance monitoring
- Type safety

Key performance optimizations:
- Direct memory access (no JSON serialization over HTTP)
- Connection reuse across requests
- Rust's zero-cost abstractions
- Async runtime efficiency
- Proper exception handling
- Performance metrics collection
"""

import asyncio
import time
import concurrent.futures
from typing import List, Dict, Any, Optional, Union
import logging

# Configure logging for better debugging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

try:
    import nexus_nitro_llm
    from nexus_nitro_llm import (
        PyConfig, PyMessage, PyLightLLMClient, PyStreamingClient,
        LightLLMError, ConnectionError, ConfigurationError
    )
    BINDINGS_AVAILABLE = True
except ImportError as e:
    logger.error(f"Failed to import nexus_nitro_llm: {e}")
    logger.error("Please install the Python bindings: pip install -e .")
    BINDINGS_AVAILABLE = False
    nexus_nitro_llm = None

class HighPerformanceLLMProcessor:
    """High-performance LLM processor using Rust bindings with comprehensive error handling."""

    def __init__(self, backend_url: str, model_id: str = "llama", token: Optional[str] = None):
        """Initialize with optimized configuration and error handling."""
        
        if not BINDINGS_AVAILABLE:
            raise ImportError("NexusNitroLLM Python bindings not available. Please install with: pip install -e .")

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

            # Create clients with error handling
            self.client = PyLightLLMClient(self.config)
            self.streaming_client = PyStreamingClient(self.config)

            logger.info(f"🚀 High-performance processor initialized")
            logger.info(f"   Backend: {backend_url}")
            logger.info(f"   Model: {model_id}")
            logger.info(f"   Connection pooling: Enabled")
            logger.info(f"   Token: {'Set' if token else 'Not set'}")

        except ConfigurationError as e:
            logger.error(f"Configuration error: {e}")
            raise
        except Exception as e:
            logger.error(f"Failed to initialize processor: {e}")
            raise

    def single_request(
        self,
        prompt: str,
        max_tokens: int = 100,
        temperature: float = 0.7
    ) -> Dict[str, Any]:
        """Send a single high-performance request."""

        messages = [nexus_nitro_llm.create_message("user", prompt)]

        start_time = time.time()
        response = self.client.chat_completions(
            messages=messages,
            max_tokens=max_tokens,
            temperature=temperature
        )
        elapsed = (time.time() - start_time) * 1000

        print(f"⚡ Direct request completed in {elapsed:.1f}ms")
        return response

    def batch_requests(
        self,
        prompts: List[str],
        max_tokens: int = 100,
        max_workers: int = 10
    ) -> List[Dict[str, Any]]:
        """Process multiple requests concurrently with connection pooling."""

        print(f"🔄 Processing {len(prompts)} requests concurrently...")
        start_time = time.time()

        def process_single(prompt: str) -> Dict[str, Any]:
            # Each thread reuses the same client (connection pooling)
            messages = [nexus_nitro_llm.create_message("user", prompt)]
            return self.client.chat_completions(
                messages=messages,
                max_tokens=max_tokens,
                temperature=0.7
            )

        # Use ThreadPoolExecutor for concurrent requests
        with concurrent.futures.ThreadPoolExecutor(max_workers=max_workers) as executor:
            responses = list(executor.map(process_single, prompts))

        elapsed = (time.time() - start_time) * 1000
        print(f"✅ Batch processing completed: {len(responses)} responses in {elapsed:.1f}ms")
        print(f"   Average per request: {elapsed/len(prompts):.1f}ms")
        print(f"   Throughput: {len(prompts)/(elapsed/1000):.1f} requests/second")

        return responses

    def streaming_request(
        self,
        prompt: str,
        max_tokens: int = 200
    ) -> Dict[str, Any]:
        """Send a streaming request for real-time responses."""

        messages = [nexus_nitro_llm.create_message("user", prompt)]

        print("🌊 Starting streaming request...")
        start_time = time.time()

        # Note: Current implementation returns full response
        # Future versions will implement true streaming
        response = self.streaming_client.stream_chat_completions(
            messages=messages,
            max_tokens=max_tokens,
            temperature=0.7
        )

        elapsed = (time.time() - start_time) * 1000
        print(f"🌊 Streaming response received in {elapsed:.1f}ms")

        return response

    def conversation_demo(self):
        """Demonstrate a multi-turn conversation with memory efficiency."""

        print("\n💬 Multi-turn conversation demo (memory optimized)")
        print("-" * 50)

        conversation_history = []

        turns = [
            "What is machine learning?",
            "Can you give me a practical example?",
            "How does this relate to neural networks?",
            "What are the main challenges?"
        ]

        total_time = 0

        for i, user_input in enumerate(turns, 1):
            print(f"\n👤 User: {user_input}")

            # Build conversation history efficiently
            messages = [nexus_nitro_llm.create_message("system", "You are a knowledgeable AI assistant.")]

            # Add conversation history
            for role, content in conversation_history:
                messages.append(nexus_nitro_llm.create_message(role, content))

            # Add current user message
            messages.append(nexus_nitro_llm.create_message("user", user_input))

            # Send request with direct memory access
            start_time = time.time()
            response = self.client.chat_completions(
                messages=messages,
                max_tokens=150,
                temperature=0.7
            )
            request_time = (time.time() - start_time) * 1000
            total_time += request_time

            # Extract assistant response (simplified)
            if isinstance(response, dict) and 'choices' in response:
                assistant_response = response['choices'][0]['message']['content']
                print(f"🤖 Assistant: {assistant_response}")

                # Add to history for next turn
                conversation_history.append(("user", user_input))
                conversation_history.append(("assistant", assistant_response))
            else:
                print(f"🤖 Assistant: [Response format: {type(response)}]")

            print(f"   ⚡ Response time: {request_time:.1f}ms")

        print(f"\n📊 Conversation statistics:")
        print(f"   Total turns: {len(turns)}")
        print(f"   Total time: {total_time:.1f}ms")
        print(f"   Average per turn: {total_time/len(turns):.1f}ms")
        print(f"   Memory efficiency: Rust zero-copy where possible")

def main():
    """Main demonstration of advanced features."""

    print("🚀 LightLLM Rust Python Bindings - Advanced Usage")
    print("=" * 60)

    # Initialize high-performance processor
    try:
        processor = HighPerformanceLLMProcessor(
            backend_url="http://localhost:8000",
            model_id="llama"
        )
    except Exception as e:
        print(f"❌ Failed to initialize processor: {e}")
        print("💡 Make sure LightLLM backend is running on localhost:8000")
        return

    # Test single request performance
    print("\n1. 🎯 Single Request Performance Test")
    print("-" * 40)
    try:
        response = processor.single_request(
            "Explain the benefits of Rust for systems programming in one sentence.",
            max_tokens=50
        )
        print("✅ Single request successful")
    except Exception as e:
        print(f"❌ Single request failed: {e}")

    # Test batch processing
    print("\n2. 🔄 Batch Processing Performance Test")
    print("-" * 40)
    prompts = [
        "What is artificial intelligence?",
        "Explain machine learning briefly.",
        "What are neural networks?",
        "Define deep learning.",
        "What is natural language processing?",
    ]

    try:
        batch_responses = processor.batch_requests(prompts, max_tokens=30)
        print(f"✅ Batch processing successful: {len(batch_responses)} responses")
    except Exception as e:
        print(f"❌ Batch processing failed: {e}")

    # Test streaming (when available)
    print("\n3. 🌊 Streaming Performance Test")
    print("-" * 40)
    try:
        stream_response = processor.streaming_request(
            "Write a short poem about the speed of light.",
            max_tokens=100
        )
        print("✅ Streaming request successful")
    except Exception as e:
        print(f"❌ Streaming request failed: {e}")

    # Conversation demo
    processor.conversation_demo()

    print("\n✨ Advanced usage demonstration complete!")
    print("\n🎯 Performance Highlights:")
    print("  • Direct Rust function calls (no HTTP overhead)")
    print("  • Connection pooling for backend efficiency")
    print("  • Concurrent request processing")
    print("  • Zero-copy data transfer optimizations")
    print("  • Memory-safe operations with Rust guarantees")
    print("  • Async runtime efficiency")

def benchmark_scaling():
    """Benchmark scaling characteristics."""

    print("\n📈 Scaling Benchmark")
    print("=" * 30)

    processor = HighPerformanceLLMProcessor("http://localhost:8000")

    # Test different batch sizes
    batch_sizes = [1, 5, 10, 20]

    for batch_size in batch_sizes:
        print(f"\nTesting batch size: {batch_size}")
        prompts = [f"Test prompt {i}" for i in range(batch_size)]

        try:
            start_time = time.time()
            responses = processor.batch_requests(prompts, max_tokens=10, max_workers=batch_size)
            total_time = (time.time() - start_time) * 1000

            print(f"  Total time: {total_time:.1f}ms")
            print(f"  Per request: {total_time/batch_size:.1f}ms")
            print(f"  Throughput: {batch_size/(total_time/1000):.1f} req/sec")

        except Exception as e:
            print(f"  Failed: {e}")

if __name__ == "__main__":
    main()

    # Uncomment to run scaling benchmark
    # benchmark_scaling()