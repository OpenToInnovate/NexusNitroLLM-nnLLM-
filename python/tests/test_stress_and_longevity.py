#!/usr/bin/env python3
"""
Stress testing and longevity tests for LightLLM Rust Python bindings.

These tests verify that the bindings work robustly under high load,
extended usage, and stress conditions typical of production environments.
"""

import pytest
import time
import threading
import concurrent.futures
import gc
import psutil
import os
import weakref
import queue
from typing import List, Dict, Any
from dataclasses import dataclass

# Import the bindings
try:
    import nexus_nitro_llm
    BINDINGS_AVAILABLE = True
except ImportError:
    BINDINGS_AVAILABLE = False
    nexus_nitro_llm = None


@dataclass
class StressTestResult:
    """Results from a stress test run."""
    duration: float
    requests_completed: int
    requests_failed: int
    average_response_time: float
    peak_memory_mb: float
    final_memory_mb: float
    errors: List[str]


class TestStressAndLongevity:
    """Stress tests for high load and long-running scenarios."""

    @pytest.fixture(autouse=True)
    def setup_method(self):
        """Set up test environment before each test."""
        if not BINDINGS_AVAILABLE:
            pytest.skip("Python bindings not available - run 'maturin develop --features python' first")

        # Record initial memory usage
        self.initial_memory = self.get_memory_usage()

    def get_memory_usage(self) -> float:
        """Get current memory usage in MB."""
        process = psutil.Process(os.getpid())
        return process.memory_info().rss / 1024 / 1024

    def test_high_volume_config_creation(self):
        """Test creating large numbers of configuration objects."""
        print("\n🔥 Testing high-volume config creation...")

        start_time = time.time()
        peak_memory = self.initial_memory
        configs = []

        # Create 10,000 configurations
        for i in range(10000):
            config = nexus_nitro_llm.PyConfig(
                backend_url=f"http://host{i % 100}.example.com:{8000 + i % 1000}",
                model_id=f"model-{i % 20}",
                port=3000 + (i % 5000)
            )
            configs.append(config)

            # Check memory every 1000 iterations
            if i % 1000 == 0:
                current_memory = self.get_memory_usage()
                peak_memory = max(peak_memory, current_memory)
                print(f"  Created {i+1:,} configs, memory: {current_memory:.1f}MB")

        elapsed = time.time() - start_time
        final_memory = self.get_memory_usage()

        print(f"✅ Created {len(configs):,} configs in {elapsed:.2f}s")
        print(f"   Rate: {len(configs)/elapsed:,.0f} configs/second")
        print(f"   Memory: {self.initial_memory:.1f}MB → {final_memory:.1f}MB (peak: {peak_memory:.1f}MB)")

        # Verify configurations
        for i, config in enumerate([configs[0], configs[5000], configs[-1]]):
            expected_url = f"http://host{i % 100 if i < len(configs) else (len(configs)-1) % 100}.example.com:{8000 + i % 1000 if i < len(configs) else (len(configs)-1) % 1000}"
            # Just check that config is accessible
            assert config.backend_url is not None
            assert config.model_id is not None

        # Performance assertions
        assert elapsed < 30.0, f"Config creation too slow: {elapsed:.2f}s"
        memory_growth = final_memory - self.initial_memory
        assert memory_growth < 500, f"Excessive memory usage: {memory_growth:.1f}MB growth"

    def test_concurrent_client_operations(self):
        """Test concurrent client operations under high load."""
        print("\n🔥 Testing concurrent client operations...")

        def worker_thread(thread_id: int, results: List[Dict]):
            """Worker thread that creates clients and performs operations."""
            thread_results = {
                'thread_id': thread_id,
                'clients_created': 0,
                'operations_completed': 0,
                'errors': []
            }

            try:
                config = nexus_nitro_llm.PyConfig(
                    backend_url=f"http://worker{thread_id}.local:8000",
                    model_id=f"worker-model-{thread_id}"
                )

                # Create multiple clients per thread
                clients = []
                for i in range(20):
                    client = nexus_nitro_llm.PyNexusNitroLLMClient(config)
                    clients.append(client)
                    thread_results['clients_created'] += 1

                # Perform operations on each client
                for client in clients:
                    for op in range(10):
                        stats = client.get_stats()
                        assert isinstance(stats, dict)
                        thread_results['operations_completed'] += 1

            except Exception as e:
                thread_results['errors'].append(str(e))

            results.append(thread_results)

        # Run concurrent threads
        start_time = time.time()
        threads = []
        results = []

        for thread_id in range(50):  # 50 concurrent threads
            thread = threading.Thread(target=worker_thread, args=(thread_id, results))
            threads.append(thread)
            thread.start()

        # Wait for completion
        for thread in threads:
            thread.join()

        elapsed = time.time() - start_time
        final_memory = self.get_memory_usage()

        # Analyze results
        total_clients = sum(r['clients_created'] for r in results)
        total_operations = sum(r['operations_completed'] for r in results)
        total_errors = sum(len(r['errors']) for r in results)

        print(f"✅ Concurrent test completed in {elapsed:.2f}s")
        print(f"   Threads: {len(threads)}")
        print(f"   Clients created: {total_clients:,}")
        print(f"   Operations completed: {total_operations:,}")
        print(f"   Errors: {total_errors}")
        print(f"   Memory: {self.initial_memory:.1f}MB → {final_memory:.1f}MB")

        # Assertions
        assert total_errors == 0, f"Errors occurred: {total_errors}"
        assert total_clients == 50 * 20, "Not all clients were created"
        assert total_operations == total_clients * 10, "Not all operations completed"

        memory_growth = final_memory - self.initial_memory
        assert memory_growth < 200, f"Excessive memory usage: {memory_growth:.1f}MB growth"

    def test_memory_leak_detection(self):
        """Test for memory leaks during repeated operations."""
        print("\n🔍 Testing for memory leaks...")

        memory_samples = []

        def sample_memory():
            gc.collect()  # Force garbage collection
            return self.get_memory_usage()

        # Baseline memory
        memory_samples.append(sample_memory())

        # Perform repeated operations that should not leak memory
        for cycle in range(10):
            print(f"  Memory leak test cycle {cycle + 1}/10...")

            # Create and destroy many objects
            for i in range(1000):
                config = nexus_nitro_llm.PyConfig(
                    backend_url=f"http://temp{i}.local:8000",
                    model_id=f"temp-{i}"
                )
                client = nexus_nitro_llm.PyNexusNitroLLMClient(config)

                # Use the objects
                stats = client.get_stats()
                assert isinstance(stats, dict)

                # Create messages
                messages = []
                for j in range(10):
                    msg = nexus_nitro_llm.create_message("user", f"Message {j}")
                    messages.append(msg)

                # Objects should be automatically cleaned up when going out of scope

            # Sample memory after each cycle
            memory_samples.append(sample_memory())

        # Analyze memory growth
        initial_memory = memory_samples[0]
        final_memory = memory_samples[-1]
        max_memory = max(memory_samples)

        print(f"✅ Memory leak test completed")
        print(f"   Initial memory: {initial_memory:.1f}MB")
        print(f"   Final memory: {final_memory:.1f}MB")
        print(f"   Peak memory: {max_memory:.1f}MB")
        print(f"   Net growth: {final_memory - initial_memory:.1f}MB")

        # Check for memory leaks (allowing some growth for Python overhead)
        memory_growth = final_memory - initial_memory
        assert memory_growth < 50, f"Potential memory leak: {memory_growth:.1f}MB growth"

    def test_long_running_stability(self):
        """Test stability over extended time periods."""
        print("\n⏰ Testing long-running stability (30 second test)...")

        # Create persistent objects
        config = nexus_nitro_llm.PyConfig(
            backend_url="http://localhost:8000",
            model_id="stability-test"
        )
        client = nexus_nitro_llm.PyNexusNitroLLMClient(config)
        streaming_client = nexus_nitro_llm.PyStreamingClient(config)

        start_time = time.time()
        operations_completed = 0
        errors = []
        memory_samples = []

        # Run for 30 seconds
        while time.time() - start_time < 30.0:
            try:
                # Perform various operations
                stats = client.get_stats()
                assert isinstance(stats, dict)

                # Create and use messages
                messages = []
                for i in range(5):
                    msg = nexus_nitro_llm.create_message(
                        "user" if i % 2 == 0 else "assistant",
                        f"Long running test message {operations_completed}-{i}"
                    )
                    messages.append(msg)

                # Test connection (this will fail but should not crash)
                client.test_connection()

                operations_completed += 1

                # Sample memory every 100 operations
                if operations_completed % 100 == 0:
                    memory_samples.append(self.get_memory_usage())
                    print(f"    Operations: {operations_completed:,}, Memory: {memory_samples[-1]:.1f}MB")

                # Small delay to avoid overwhelming the system
                time.sleep(0.001)

            except Exception as e:
                errors.append(str(e))

        elapsed = time.time() - start_time
        final_memory = self.get_memory_usage()

        print(f"✅ Long-running test completed")
        print(f"   Duration: {elapsed:.1f}s")
        print(f"   Operations: {operations_completed:,}")
        print(f"   Rate: {operations_completed/elapsed:.0f} ops/second")
        print(f"   Errors: {len(errors)}")
        print(f"   Final memory: {final_memory:.1f}MB")

        # Stability assertions
        assert operations_completed > 1000, f"Not enough operations completed: {operations_completed}"
        assert len(errors) == 0, f"Errors occurred: {errors[:5]}"  # Show first 5 errors

        # Memory should be stable
        if memory_samples:
            memory_growth = memory_samples[-1] - memory_samples[0]
            assert abs(memory_growth) < 20, f"Memory instability: {memory_growth:.1f}MB change"

    def test_thread_safety_stress(self):
        """Test thread safety under extreme concurrent access."""
        print("\n🧵 Testing thread safety under stress...")

        # Shared resources
        config = nexus_nitro_llm.PyConfig(
            backend_url="http://shared.local:8000",
            model_id="thread-safety-test"
        )

        results_queue = queue.Queue()
        barrier = threading.Barrier(20)  # Synchronize 20 threads

        def stress_worker(worker_id: int):
            """Worker that performs many operations concurrently."""
            try:
                # Wait for all threads to be ready
                barrier.wait()

                # Create client (this should be thread-safe)
                client = nexus_nitro_llm.PyNexusNitroLLMClient(config)

                operations = 0
                errors = []

                # Perform rapid operations
                for i in range(500):
                    try:
                        # Rapid operations that test thread safety
                        stats = client.get_stats()
                        assert isinstance(stats, dict)

                        # Create messages rapidly
                        msg = nexus_nitro_llm.create_message("user", f"Worker {worker_id} message {i}")
                        assert msg.content == f"Worker {worker_id} message {i}"

                        operations += 1

                    except Exception as e:
                        errors.append(str(e))

                results_queue.put({
                    'worker_id': worker_id,
                    'operations': operations,
                    'errors': errors
                })

            except Exception as e:
                results_queue.put({
                    'worker_id': worker_id,
                    'operations': 0,
                    'errors': [f"Fatal error: {e}"]
                })

        # Start all threads
        start_time = time.time()
        threads = []

        for i in range(20):
            thread = threading.Thread(target=stress_worker, args=(i,))
            threads.append(thread)
            thread.start()

        # Wait for completion
        for thread in threads:
            thread.join()

        elapsed = time.time() - start_time
        final_memory = self.get_memory_usage()

        # Collect results
        results = []
        while not results_queue.empty():
            results.append(results_queue.get())

        total_operations = sum(r['operations'] for r in results)
        total_errors = sum(len(r['errors']) for r in results)

        print(f"✅ Thread safety stress test completed")
        print(f"   Duration: {elapsed:.2f}s")
        print(f"   Threads: {len(threads)}")
        print(f"   Total operations: {total_operations:,}")
        print(f"   Rate: {total_operations/elapsed:,.0f} ops/second")
        print(f"   Errors: {total_errors}")
        print(f"   Memory: {final_memory:.1f}MB")

        # Thread safety assertions
        assert len(results) == 20, "Not all threads completed"
        assert total_errors == 0, f"Thread safety errors: {total_errors}"
        assert total_operations == 20 * 500, "Not all operations completed"

    def test_resource_cleanup_stress(self):
        """Test that resources are properly cleaned up under stress."""
        print("\n🧹 Testing resource cleanup under stress...")

        # Create many objects and let them go out of scope
        weak_refs = []

        for cycle in range(100):
            objects_in_cycle = []

            # Create many objects
            for i in range(100):
                config = nexus_nitro_llm.PyConfig(
                    backend_url=f"http://cleanup{i}.test:8000",
                    model_id=f"cleanup-{i}"
                )
                client = nexus_nitro_llm.PyNexusNitroLLMClient(config)
                messages = [
                    nexus_nitro_llm.create_message("user", f"Cleanup test {cycle}-{i}")
                    for _ in range(5)
                ]

                objects_in_cycle.extend([config, client] + messages)

            # Keep weak references to detect cleanup
            cycle_refs = [weakref.ref(obj) for obj in objects_in_cycle]
            weak_refs.extend(cycle_refs)

            # Objects should be cleaned up when going out of scope
            del objects_in_cycle

            if cycle % 20 == 0:
                gc.collect()
                memory = self.get_memory_usage()
                print(f"    Cycle {cycle + 1}/100, Memory: {memory:.1f}MB")

        # Force final cleanup
        gc.collect()
        final_memory = self.get_memory_usage()

        # Check that objects were cleaned up
        live_objects = sum(1 for ref in weak_refs if ref() is not None)
        cleanup_rate = (len(weak_refs) - live_objects) / len(weak_refs) * 100

        print(f"✅ Resource cleanup test completed")
        print(f"   Total objects created: {len(weak_refs):,}")
        print(f"   Objects cleaned up: {len(weak_refs) - live_objects:,} ({cleanup_rate:.1f}%)")
        print(f"   Live objects remaining: {live_objects:,}")
        print(f"   Final memory: {final_memory:.1f}MB")

        # Cleanup assertions
        assert cleanup_rate > 95, f"Poor cleanup rate: {cleanup_rate:.1f}%"

        memory_growth = final_memory - self.initial_memory
        assert memory_growth < 100, f"Excessive memory after cleanup: {memory_growth:.1f}MB"


if __name__ == "__main__":
    # Run stress tests when executed directly
    import sys
    if len(sys.argv) > 1 and sys.argv[1] == "stress":
        # Run just the stress tests
        pytest.main([__file__ + "::TestStressAndLongevity", "-v", "-s"])
    else:
        pytest.main([__file__, "-v"])