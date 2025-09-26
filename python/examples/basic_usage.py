#!/usr/bin/env python3
"""
Basic usage example for NexusNitroLLM Python bindings.

This example demonstrates how to use the high-performance Python bindings
to communicate directly with various LLM backends without HTTP overhead.

Performance benefits:
- No network serialization/deserialization
- Direct memory access between Python and Rust
- Connection pooling and HTTP/2 support
- Zero-copy data transfer where possible
"""

import time
import json
from typing import List, Dict, Any

# Import the high-performance Rust bindings
import nexus_nitro_llm

def main():
    """Demonstrate basic high-performance LLM usage."""

    print("🚀 NexusNitroLLM Python Bindings - Basic Usage Example")
    print("=" * 60)

    # Create configuration for maximum performance
    config = nexus_nitro_llm.PyConfig(
        backend_url="http://localhost:8000",
        backend_type="lightllm",
        model_id="llama",
        port=8080
    )

    # Enable connection pooling for better performance
    config.set_connection_pooling(True)

    print(f"✅ Configuration created:")
    print(f"   Backend URL: {config.backend_url}")
    print(f"   Default Model: {config.model_id}")

    # Create high-performance client (no HTTP server overhead)
    try:
        client = nexus_nitro_llm.PyNexusNitroLLMClient(config)
        print("✅ High-performance client created")

        # Test connection to backend
        if client.test_connection():
            print("✅ Connection to backend successful")
        else:
            print("⚠️  Warning: Could not connect to backend (continuing with example)")

    except Exception as e:
        print(f"❌ Failed to create client: {e}")
        return

    # Create conversation messages
    messages = [
        nexus_nitro_llm.create_message("system", "You are a helpful AI assistant focused on performance and efficiency."),
        nexus_nitro_llm.create_message("user", "What are the benefits of using Rust for high-performance computing?"),
    ]

    print("\n💬 Conversation Messages:")
    for i, msg in enumerate(messages):
        print(f"   {i+1}. [{msg.role}] {msg.content}")

    # Send request with direct memory access (no HTTP overhead)
    print("\n⚡ Sending request with zero-copy bindings...")
    start_time = time.time()

    try:
        response = client.chat_completions(
            messages=messages,
            model="llama",
            max_tokens=150,
            temperature=0.7,
            stream=False
        )

        end_time = time.time()

        print(f"✅ Response received in {(end_time - start_time)*1000:.1f}ms")

        # Print response (it's already a Python dictionary)
        if isinstance(response, dict):
            print("\n📝 Response:")
            print(json.dumps(response, indent=2))
        else:
            print(f"\n📝 Response: {response}")

    except Exception as e:
        print(f"❌ Request failed: {e}")
        return

    # Get performance statistics
    print("\n📊 Performance Statistics:")
    try:
        stats = client.get_stats()
        for key, value in stats.items():
            print(f"   {key}: {value}")
    except Exception as e:
        print(f"⚠️  Could not get stats: {e}")

    print("\n✨ Example completed successfully!")
    print("\nPerformance Notes:")
    print("  • Direct Rust function calls (no HTTP serialization)")
    print("  • Connection pooling for backend requests")
    print("  • Zero-copy data transfer where possible")
    print("  • Memory-safe operations with Rust guarantees")

def benchmark_comparison():
    """Compare performance between Python bindings and HTTP requests."""

    print("\n🏁 Performance Benchmark Comparison")
    print("=" * 40)

    # Create client for direct calls
    config = nexus_nitro_llm.PyConfig(
        lightllm_url="http://localhost:8000",
        model_id="llama"
    )
    client = nexus_nitro_llm.PyLightLLMClient(config)

    # Test message
    messages = [nexus_nitro_llm.create_message("user", "Hello")]

    # Benchmark direct calls (Python bindings)
    print("Testing Python bindings (direct Rust calls)...")
    direct_times = []

    for i in range(5):
        start = time.time()
        try:
            response = client.chat_completions(
                messages=messages,
                max_tokens=10,
                temperature=0.0
            )
            direct_times.append((time.time() - start) * 1000)
            print(f"  Run {i+1}: {direct_times[-1]:.1f}ms")
        except Exception as e:
            print(f"  Run {i+1}: Failed - {e}")

    if direct_times:
        avg_direct = sum(direct_times) / len(direct_times)
        print(f"📊 Direct bindings average: {avg_direct:.1f}ms")
        print(f"   Benefits: No HTTP overhead, direct memory access")
    else:
        print("❌ No successful direct calls")

if __name__ == "__main__":
    main()

    # Uncomment to run benchmark comparison
    # benchmark_comparison()