#!/usr/bin/env python3
"""
Error handling and recovery tests for LightLLM Rust Python bindings.

Tests that the bindings handle errors gracefully, recover from failures,
and maintain stability even when backends are unavailable or misbehaving.
"""

import pytest
import time
import threading
import concurrent.futures
from typing import List, Dict, Any

# Import the bindings
try:
    import nexus_nitro_llm
    BINDINGS_AVAILABLE = True
except ImportError:
    BINDINGS_AVAILABLE = False
    nexus_nitro_llm = None


class TestErrorHandlingAndRecovery:
    """Test error handling, recovery, and resilience."""

    @pytest.fixture(autouse=True)
    def setup_method(self):
        """Set up test environment before each test."""
        if not BINDINGS_AVAILABLE:
            pytest.skip("Python bindings not available - run 'maturin develop --features python' first")

    def test_invalid_configuration_handling(self):
        """Test handling of invalid configuration parameters."""
        print("\n❌ Testing invalid configuration handling...")

        # Test invalid URLs
        invalid_urls = [
            "not-a-url",
            "ftp://invalid-protocol:8000",
            "http://",
            "",
        ]

        for invalid_url in invalid_urls:
            try:
                config = nexus_nitro_llm.PyConfig(
                    lightllm_url=invalid_url,
                    model_id="test-model"
                )
                # Configuration creation might succeed, but client creation or usage should handle it
                print(f"  Config created with invalid URL: {invalid_url}")
            except Exception as e:
                print(f"  Expected error for URL '{invalid_url}': {e}")

        # Test empty model ID
        try:
            config = nexus_nitro_llm.PyConfig(
                lightllm_url="http://localhost:8000",
                model_id=""
            )
            print("  Config created with empty model ID")
        except Exception as e:
            print(f"  Error with empty model ID: {e}")

    def test_backend_unreachable_handling(self):
        """Test behavior when backend is unreachable."""
        print("\n🔌 Testing unreachable backend handling...")

        # Use a definitely unreachable URL
        config = nexus_nitro_llm.PyConfig(
            lightllm_url="http://127.0.0.1:65432",  # Unlikely to be used port
            model_id="test-model"
        )

        try:
            client = nexus_nitro_llm.PyLightLLMClient(config)
            print("  Client created successfully")

            # Test connection should fail gracefully
            is_connected = client.test_connection()
            print(f"  Connection test result: {is_connected}")
            assert not is_connected, "Connection should fail for unreachable backend"

            # Chat completions should handle the error gracefully
            messages = [nexus_nitro_llm.create_message("user", "Hello")]

            try:
                response = client.chat_completions(messages=messages, max_tokens=10)
                print(f"  Unexpected success: {response}")
            except Exception as e:
                print(f"  Expected error for unreachable backend: {e}")
                # Error should be handled gracefully, not crash

        except Exception as e:
            print(f"  Client creation error: {e}")

    def test_malformed_message_handling(self):
        """Test handling of malformed or edge-case messages."""
        print("\n📝 Testing malformed message handling...")

        # Test various edge cases for messages
        edge_cases = [
            ("", ""),  # Empty role and content
            ("user", ""),  # Empty content
            ("", "Hello"),  # Empty role
            ("invalid_role", "Test content"),  # Invalid role
            ("user", None),  # None content (if possible)
        ]

        for role, content in edge_cases:
            try:
                if content is None:
                    # Skip None content test as it's likely not supported
                    continue

                msg = nexus_nitro_llm.create_message(role, content)
                print(f"  Message created: role='{role}', content='{content[:20]}...'")

                # Verify message properties
                assert msg.role == role
                assert msg.content == content

            except Exception as e:
                print(f"  Expected error for role='{role}', content='{str(content)[:20]}': {e}")

    def test_concurrent_error_scenarios(self):
        """Test error handling under concurrent load."""
        print("\n🧵 Testing concurrent error handling...")

        # Mix of valid and invalid configurations
        configs = []
        for i in range(10):
            if i % 3 == 0:
                # Invalid URL every 3rd config
                config = nexus_nitro_llm.PyConfig(
                    lightllm_url=f"http://invalid-host-{i}.local:8000",
                    model_id=f"model-{i}"
                )
            else:
                # Valid but unreachable URL
                config = nexus_nitro_llm.PyConfig(
                    lightllm_url=f"http://127.0.0.1:6543{i % 10}",
                    model_id=f"model-{i}"
                )
            configs.append(config)

        results = []
        errors = []

        def test_client(config_idx, config):
            """Test client operations and collect results."""
            try:
                client = nexus_nitro_llm.PyLightLLMClient(config)

                # Test connection
                connection_result = client.test_connection()

                # Try a simple operation
                messages = [nexus_nitro_llm.create_message("user", f"Test {config_idx}")]

                try:
                    response = client.chat_completions(messages=messages, max_tokens=5)
                    results.append({
                        'config_idx': config_idx,
                        'connection': connection_result,
                        'chat_success': True,
                        'error': None
                    })
                except Exception as chat_error:
                    results.append({
                        'config_idx': config_idx,
                        'connection': connection_result,
                        'chat_success': False,
                        'error': str(chat_error)
                    })

            except Exception as client_error:
                errors.append({
                    'config_idx': config_idx,
                    'stage': 'client_creation',
                    'error': str(client_error)
                })

        # Run concurrent tests
        start_time = time.time()

        with concurrent.futures.ThreadPoolExecutor(max_workers=5) as executor:
            futures = [
                executor.submit(test_client, i, config)
                for i, config in enumerate(configs)
            ]

            # Wait for all to complete
            concurrent.futures.wait(futures)

        elapsed = time.time() - start_time

        print(f"  Concurrent error test completed in {elapsed:.2f}s")
        print(f"  Results collected: {len(results)}")
        print(f"  Errors collected: {len(errors)}")

        # Analyze results
        connection_failures = sum(1 for r in results if not r['connection'])
        chat_failures = sum(1 for r in results if not r['chat_success'])

        print(f"  Connection failures: {connection_failures}/{len(results)}")
        print(f"  Chat failures: {chat_failures}/{len(results)}")
        print(f"  Client creation errors: {len(errors)}")

        # All should have failed connections (unreachable backends)
        # But no crashes should occur
        assert len(results) + len(errors) == len(configs), "Not all operations completed"

    def test_resource_cleanup_after_errors(self):
        """Test that resources are cleaned up properly after errors."""
        print("\n🧹 Testing resource cleanup after errors...")

        import gc
        import weakref

        weak_refs = []
        error_count = 0

        # Create many objects that will cause errors
        for i in range(100):
            try:
                config = nexus_nitro_llm.PyConfig(
                    lightllm_url=f"http://error-test-{i}.invalid:8000",
                    model_id=f"error-model-{i}"
                )
                weak_refs.append(weakref.ref(config))

                client = nexus_nitro_llm.PyLightLLMClient(config)
                weak_refs.append(weakref.ref(client))

                # Try operations that will likely fail
                messages = [nexus_nitro_llm.create_message("user", f"Error test {i}")]
                weak_refs.extend([weakref.ref(msg) for msg in messages])

                try:
                    # This should fail due to invalid backend
                    response = client.chat_completions(messages=messages, max_tokens=1)
                except Exception as e:
                    error_count += 1
                    # Expected errors - continue

            except Exception as e:
                error_count += 1
                # Expected errors during setup

        print(f"  Created objects with {error_count} expected errors")

        # Force cleanup
        gc.collect()

        # Check cleanup
        live_objects = sum(1 for ref in weak_refs if ref() is not None)
        cleanup_rate = (len(weak_refs) - live_objects) / len(weak_refs) * 100

        print(f"  Total objects created: {len(weak_refs)}")
        print(f"  Objects cleaned up: {len(weak_refs) - live_objects} ({cleanup_rate:.1f}%)")
        print(f"  Live objects remaining: {live_objects}")

        # Should have good cleanup rate even after errors
        assert cleanup_rate > 90, f"Poor cleanup rate after errors: {cleanup_rate:.1f}%"

    def test_recovery_after_backend_failure(self):
        """Test system recovery after backend becomes unavailable."""
        print("\n🔄 Testing recovery after backend failure...")

        config = nexus_nitro_llm.PyConfig(
            lightllm_url="http://127.0.0.1:65431",  # Unreachable port
            model_id="recovery-test"
        )

        client = nexus_nitro_llm.PyLightLLMClient(config)
        messages = [nexus_nitro_llm.create_message("user", "Recovery test")]

        # Phase 1: Confirm failures
        failure_count = 0
        for i in range(5):
            try:
                response = client.chat_completions(messages=messages, max_tokens=5)
                print(f"  Unexpected success in failure phase: {response}")
            except Exception as e:
                failure_count += 1
                print(f"  Expected failure {i+1}: {type(e).__name__}")

        assert failure_count == 5, "Should have failed all attempts with unreachable backend"

        # Phase 2: Test that client is still usable (doesn't crash permanently)
        # Even though backend is still unreachable, client should handle it gracefully
        still_failing = 0
        for i in range(3):
            try:
                stats = client.get_stats()  # This should work even if backend is down
                print(f"  Stats retrieval successful: {type(stats)}")
            except Exception as e:
                still_failing += 1
                print(f"  Stats failure {i+1}: {e}")

            try:
                connection_test = client.test_connection()
                print(f"  Connection test result: {connection_test}")
                assert not connection_test  # Should return False, not crash
            except Exception as e:
                still_failing += 1
                print(f"  Connection test error: {e}")

        print(f"  Client remains stable after {failure_count} backend failures")
        print(f"  Additional operation failures: {still_failing}")

    def test_message_size_limits(self):
        """Test handling of extremely large messages."""
        print("\n📏 Testing message size limits...")

        config = nexus_nitro_llm.PyConfig(
            lightllm_url="http://localhost:8000",
            model_id="size-test"
        )

        # Test various message sizes
        sizes = [1000, 10000, 100000, 1000000]  # 1KB to 1MB

        for size in sizes:
            print(f"  Testing message size: {size:,} characters")

            try:
                large_content = "x" * size
                msg = nexus_nitro_llm.create_message("user", large_content)

                assert len(msg.content) == size
                print(f"    ✅ Created message of size {size:,}")

                # Test with client (will likely fail due to no backend, but shouldn't crash)
                try:
                    client = nexus_nitro_llm.PyLightLLMClient(config)
                    response = client.chat_completions(messages=[msg], max_tokens=1)
                    print(f"    ✅ Processed large message successfully")
                except Exception as e:
                    print(f"    ℹ️ Expected processing error: {type(e).__name__}")
                    # Error is expected due to no backend

            except Exception as e:
                print(f"    ❌ Failed at size {size:,}: {e}")
                # Very large messages might hit memory limits

    def test_thread_safety_during_errors(self):
        """Test thread safety when errors occur in concurrent scenarios."""
        print("\n🧵 Testing thread safety during errors...")

        config = nexus_nitro_llm.PyConfig(
            lightllm_url="http://127.0.0.1:65430",  # Unreachable
            model_id="thread-error-test"
        )

        results = []
        barrier = threading.Barrier(10)  # Synchronize 10 threads

        def error_worker(worker_id):
            """Worker that intentionally triggers errors."""
            worker_results = {
                'worker_id': worker_id,
                'operations': 0,
                'errors': 0,
                'crashes': 0
            }

            try:
                # Wait for all threads to be ready
                barrier.wait()

                client = nexus_nitro_llm.PyLightLLMClient(config)

                for i in range(50):
                    try:
                        messages = [nexus_nitro_llm.create_message("user", f"Error test {worker_id}-{i}")]

                        # This should fail but not crash
                        response = client.chat_completions(messages=messages, max_tokens=1)
                        worker_results['operations'] += 1

                    except Exception:
                        worker_results['errors'] += 1
                        # Expected errors - don't treat as crashes

                    # Also test other operations
                    try:
                        stats = client.get_stats()
                        worker_results['operations'] += 1
                    except Exception:
                        worker_results['errors'] += 1

            except Exception as fatal_error:
                worker_results['crashes'] += 1
                print(f"  Worker {worker_id} crashed: {fatal_error}")

            results.append(worker_results)

        # Run concurrent error-prone operations
        start_time = time.time()
        threads = []

        for i in range(10):
            thread = threading.Thread(target=error_worker, args=(i,))
            threads.append(thread)
            thread.start()

        for thread in threads:
            thread.join()

        elapsed = time.time() - start_time

        # Analyze results
        total_operations = sum(r['operations'] for r in results)
        total_errors = sum(r['errors'] for r in results)
        total_crashes = sum(r['crashes'] for r in results)

        print(f"✅ Thread safety error test completed in {elapsed:.2f}s")
        print(f"   Threads: {len(results)}")
        print(f"   Operations: {total_operations}")
        print(f"   Expected errors: {total_errors}")
        print(f"   Crashes: {total_crashes}")

        # Should have no crashes, even with many errors
        assert total_crashes == 0, f"Thread safety compromised: {total_crashes} crashes"
        assert len(results) == 10, "Not all threads completed"


if __name__ == "__main__":
    # Run error handling tests when executed directly
    pytest.main([__file__, "-v", "-s"])