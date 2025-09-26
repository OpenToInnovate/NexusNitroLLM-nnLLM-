#!/usr/bin/env python3
"""
Mode Comparison Example for LightLLM Rust Python bindings.

This example demonstrates the difference between HTTP mode and Direct mode,
showing when to use each approach and their performance characteristics.

Modes:
1. HTTP Mode: Traditional proxy approach with HTTP communication
2. Direct Mode: Direct integration without HTTP overhead
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


class ModeComparison:
    """Compare HTTP mode vs Direct mode performance and usage."""

    def __init__(self):
        """Initialize both modes for comparison."""
        
        if not BINDINGS_AVAILABLE:
            raise ImportError("LightLLM Rust bindings not available. Please install with: pip install -e .")

        # HTTP Mode Configuration
        self.http_config = PyConfig(
            lightllm_url="http://localhost:8000",  # Traditional HTTP endpoint
            model_id="llama",
            token=None,
            timeout=30
        )
        self.http_config.set_connection_pooling(True)
        self.http_client = PyLightLLMClient(self.http_config)
        self.http_async_client = PyAsyncLightLLMClient(self.http_config)

        # Direct Mode Configuration
        self.direct_config = PyConfig(
            # No URL needed - defaults to direct mode
            model_id="llama",
            token=None,
            timeout=30
        )
        self.direct_client = PyLightLLMClient(self.direct_config)
        self.direct_async_client = PyAsyncLightLLMClient(self.direct_config)

        logger.info("🚀 Mode comparison initialized")
        logger.info("   HTTP Mode: Traditional proxy with HTTP communication")
        logger.info("   Direct Mode: Direct integration without HTTP overhead")

    def test_http_mode(self, prompt: str, max_tokens: int = 50) -> Dict[str, Any]:
        """Test HTTP mode performance."""
        
        messages = [nexus_nitro_llm.create_message("user", prompt)]
        
        start_time = time.time()
        response = self.http_client.chat_completions(
            messages=messages,
            max_tokens=max_tokens,
            temperature=0.7
        )
        elapsed = (time.time() - start_time) * 1000
        
        logger.info(f"🌐 HTTP mode request completed in {elapsed:.1f}ms")
        return response

    def test_direct_mode(self, prompt: str, max_tokens: int = 50) -> Dict[str, Any]:
        """Test Direct mode performance."""
        
        messages = [nexus_nitro_llm.create_message("user", prompt)]
        
        start_time = time.time()
        response = self.direct_client.chat_completions(
            messages=messages,
            max_tokens=max_tokens,
            temperature=0.7
        )
        elapsed = (time.time() - start_time) * 1000
        
        logger.info(f"⚡ Direct mode request completed in {elapsed:.1f}ms")
        return response

    async def test_async_http_mode(self, prompt: str, max_tokens: int = 50) -> Dict[str, Any]:
        """Test async HTTP mode performance."""
        
        messages = [nexus_nitro_llm.create_message("user", prompt)]
        
        start_time = time.time()
        response = await self.http_async_client.chat_completions_async(
            messages=messages,
            max_tokens=max_tokens,
            temperature=0.7
        )
        elapsed = (time.time() - start_time) * 1000
        
        logger.info(f"🌐 Async HTTP mode request completed in {elapsed:.1f}ms")
        return response

    async def test_async_direct_mode(self, prompt: str, max_tokens: int = 50) -> Dict[str, Any]:
        """Test async Direct mode performance."""
        
        messages = [nexus_nitro_llm.create_message("user", prompt)]
        
        start_time = time.time()
        response = await self.direct_async_client.chat_completions_async(
            messages=messages,
            max_tokens=max_tokens,
            temperature=0.7
        )
        elapsed = (time.time() - start_time) * 1000
        
        logger.info(f"⚡ Async Direct mode request completed in {elapsed:.1f}ms")
        return response

    def performance_comparison(self):
        """Compare performance between HTTP and Direct modes."""
        
        logger.info("\n📈 Performance Comparison: HTTP vs Direct Mode")
        logger.info("=" * 60)
        
        prompts = [
            "What is artificial intelligence?",
            "Explain machine learning briefly.",
            "What are neural networks?",
            "Define deep learning.",
            "What is natural language processing?",
        ]
        
        # Test HTTP mode
        logger.info("\n🌐 Testing HTTP Mode...")
        http_times = []
        for prompt in prompts:
            start_time = time.time()
            try:
                self.test_http_mode(prompt, max_tokens=30)
                elapsed = (time.time() - start_time) * 1000
                http_times.append(elapsed)
            except Exception as e:
                logger.warning(f"HTTP mode request failed: {e}")
                http_times.append(float('inf'))
        
        # Test Direct mode
        logger.info("\n⚡ Testing Direct Mode...")
        direct_times = []
        for prompt in prompts:
            start_time = time.time()
            try:
                self.test_direct_mode(prompt, max_tokens=30)
                elapsed = (time.time() - start_time) * 1000
                direct_times.append(elapsed)
            except Exception as e:
                logger.warning(f"Direct mode request failed: {e}")
                direct_times.append(float('inf'))
        
        # Calculate statistics
        http_avg = sum(t for t in http_times if t != float('inf')) / len([t for t in http_times if t != float('inf')])
        direct_avg = sum(t for t in direct_times if t != float('inf')) / len([t for t in direct_times if t != float('inf')])
        
        logger.info(f"\n📊 Performance Results:")
        logger.info(f"   HTTP Mode Average: {http_avg:.1f}ms")
        logger.info(f"   Direct Mode Average: {direct_avg:.1f}ms")
        if direct_avg < http_avg:
            speedup = http_avg / direct_avg
            logger.info(f"   Direct Mode Speedup: {speedup:.1f}x faster")
        else:
            logger.info(f"   HTTP Mode is faster (likely due to network conditions)")

    async def async_performance_comparison(self):
        """Compare async performance between HTTP and Direct modes."""
        
        logger.info("\n📈 Async Performance Comparison: HTTP vs Direct Mode")
        logger.info("=" * 60)
        
        prompts = [
            "What is artificial intelligence?",
            "Explain machine learning briefly.",
            "What are neural networks?",
            "Define deep learning.",
            "What is natural language processing?",
        ]
        
        # Test async HTTP mode
        logger.info("\n🌐 Testing Async HTTP Mode...")
        start_time = time.time()
        try:
            http_tasks = [self.test_async_http_mode(prompt, max_tokens=30) for prompt in prompts]
            http_responses = await asyncio.gather(*http_tasks, return_exceptions=True)
            http_elapsed = (time.time() - start_time) * 1000
            http_successful = len([r for r in http_responses if not isinstance(r, Exception)])
            logger.info(f"   HTTP Mode: {http_successful} responses in {http_elapsed:.1f}ms")
        except Exception as e:
            logger.warning(f"Async HTTP mode failed: {e}")
            http_elapsed = float('inf')
        
        # Test async Direct mode
        logger.info("\n⚡ Testing Async Direct Mode...")
        start_time = time.time()
        try:
            direct_tasks = [self.test_async_direct_mode(prompt, max_tokens=30) for prompt in prompts]
            direct_responses = await asyncio.gather(*direct_tasks, return_exceptions=True)
            direct_elapsed = (time.time() - start_time) * 1000
            direct_successful = len([r for r in direct_responses if not isinstance(r, Exception)])
            logger.info(f"   Direct Mode: {direct_successful} responses in {direct_elapsed:.1f}ms")
        except Exception as e:
            logger.warning(f"Async Direct mode failed: {e}")
            direct_elapsed = float('inf')
        
        # Calculate statistics
        if direct_elapsed < http_elapsed and direct_elapsed != float('inf'):
            speedup = http_elapsed / direct_elapsed
            logger.info(f"\n📊 Async Performance Results:")
            logger.info(f"   Direct Mode Speedup: {speedup:.1f}x faster")
        else:
            logger.info(f"\n📊 Async Performance Results:")
            logger.info(f"   Both modes performed similarly")

    def get_mode_statistics(self):
        """Get statistics for both modes."""
        
        logger.info("\n📊 Mode Statistics:")
        logger.info("-" * 30)
        
        # HTTP Mode stats
        http_stats = self.http_client.get_stats()
        logger.info(f"🌐 HTTP Mode:")
        logger.info(f"   Adapter type: {http_stats.get('adapter_type', 'unknown')}")
        logger.info(f"   Backend URL: {http_stats.get('backend_url', 'unknown')}")
        logger.info(f"   Connection pooling: {http_stats.get('connection_pooling', False)}")
        logger.info(f"   Total requests: {http_stats.get('total_requests', 0)}")
        
        # Direct Mode stats
        direct_stats = self.direct_client.get_stats()
        logger.info(f"⚡ Direct Mode:")
        logger.info(f"   Adapter type: {direct_stats.get('adapter_type', 'unknown')}")
        logger.info(f"   Backend URL: {direct_stats.get('backend_url', 'unknown')}")
        logger.info(f"   Connection pooling: {direct_stats.get('connection_pooling', False)}")
        logger.info(f"   Total requests: {direct_stats.get('total_requests', 0)}")

    def usage_recommendations(self):
        """Provide usage recommendations for each mode."""
        
        logger.info("\n💡 Usage Recommendations:")
        logger.info("=" * 40)
        
        logger.info("🌐 Use HTTP Mode when:")
        logger.info("  • You have a running LightLLM server")
        logger.info("  • You need to share the same backend across multiple applications")
        logger.info("  • You want to use existing LightLLM infrastructure")
        logger.info("  • You need to scale horizontally across multiple servers")
        logger.info("  • You want to use LightLLM's built-in features (batching, routing, etc.)")
        
        logger.info("\n⚡ Use Direct Mode when:")
        logger.info("  • You want maximum performance with minimal latency")
        logger.info("  • You're building a Python application that needs direct integration")
        logger.info("  • You don't want to run a separate LightLLM server")
        logger.info("  • You're building embedded applications")
        logger.info("  • You need zero network overhead")
        logger.info("  • You're doing high-performance computing or real-time applications")


async def main():
    """Main mode comparison demonstration."""
    
    logger.info("🚀 LightLLM Rust Python Bindings - Mode Comparison")
    logger.info("=" * 70)
    
    # Initialize mode comparison
    try:
        comparison = ModeComparison()
    except Exception as e:
        logger.error(f"❌ Failed to initialize mode comparison: {e}")
        return
    
    # Test single requests
    logger.info("\n1. 🎯 Single Request Comparison")
    logger.info("-" * 40)
    
    prompt = "Explain the difference between HTTP and Direct modes in one sentence."
    
    try:
        logger.info("Testing HTTP mode...")
        http_response = comparison.test_http_mode(prompt, max_tokens=50)
        logger.info("✅ HTTP mode request successful")
    except Exception as e:
        logger.error(f"❌ HTTP mode request failed: {e}")
    
    try:
        logger.info("Testing Direct mode...")
        direct_response = comparison.test_direct_mode(prompt, max_tokens=50)
        logger.info("✅ Direct mode request successful")
    except Exception as e:
        logger.error(f"❌ Direct mode request failed: {e}")
    
    # Test async requests
    logger.info("\n2. ⚡ Async Request Comparison")
    logger.info("-" * 40)
    
    try:
        logger.info("Testing async HTTP mode...")
        http_async_response = await comparison.test_async_http_mode(prompt, max_tokens=50)
        logger.info("✅ Async HTTP mode request successful")
    except Exception as e:
        logger.error(f"❌ Async HTTP mode request failed: {e}")
    
    try:
        logger.info("Testing async Direct mode...")
        direct_async_response = await comparison.test_async_direct_mode(prompt, max_tokens=50)
        logger.info("✅ Async Direct mode request successful")
    except Exception as e:
        logger.error(f"❌ Async Direct mode request failed: {e}")
    
    # Performance comparison
    comparison.performance_comparison()
    
    # Async performance comparison
    await comparison.async_performance_comparison()
    
    # Get statistics
    comparison.get_mode_statistics()
    
    # Usage recommendations
    comparison.usage_recommendations()
    
    logger.info("\n✨ Mode comparison demonstration complete!")
    logger.info("\n🎯 Key Takeaways:")
    logger.info("  • Direct mode provides maximum performance with zero HTTP overhead")
    logger.info("  • HTTP mode is better for shared infrastructure and scaling")
    logger.info("  • Choose the mode that best fits your use case")
    logger.info("  • Both modes support async operations for concurrent processing")


if __name__ == "__main__":
    # Run the mode comparison demonstration
    asyncio.run(main())
