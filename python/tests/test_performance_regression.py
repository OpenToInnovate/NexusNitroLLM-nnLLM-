#!/usr/bin/env python3
"""
Performance regression tests for LightLLM Rust Python bindings.

These tests establish performance baselines and detect regressions in key
performance metrics like memory usage, object creation speed, and throughput.
"""

import pytest
import time
import gc
import psutil
import os
import statistics
from typing import List, Dict, Any, Tuple
from dataclasses import dataclass

# Import the bindings
try:
    import nexus_nitro_llm
    BINDINGS_AVAILABLE = True
except ImportError:
    BINDINGS_AVAILABLE = False
    nexus_nitro_llm = None


@dataclass
class PerformanceBaseline:
    """Expected performance baselines."""
    config_creation_per_sec: int = 10000  # configs/second
    message_creation_per_sec: int = 20000  # messages/second
    client_creation_per_sec: int = 100     # clients/second
    max_memory_growth_mb: float = 100.0    # MB
    stats_retrieval_per_sec: int = 5000    # stats calls/second


@dataclass
class PerformanceResult:
    """Results from a performance test."""
    operation: str
    rate_per_second: float
    memory_growth_mb: float
    duration_seconds: float
    items_processed: int
    passed: bool
    details: Dict[str, Any]


class TestPerformanceRegression:
    """Performance regression tests."""

    @pytest.fixture(autouse=True)
    def setup_method(self):
        """Set up test environment before each test."""
        if not BINDINGS_AVAILABLE:
            pytest.skip("Python bindings not available - run 'maturin develop --features python' first")

        # Record initial memory and force cleanup
        gc.collect()
        self.initial_memory = self.get_memory_usage()
        self.baselines = PerformanceBaseline()

    def get_memory_usage(self) -> float:
        """Get current memory usage in MB."""
        process = psutil.Process(os.getpid())
        return process.memory_info().rss / 1024 / 1024

    def measure_performance(self, operation_name: str, operation_func, count: int) -> PerformanceResult:
        """Measure performance of an operation."""
        # Prepare
        gc.collect()
        start_memory = self.get_memory_usage()

        # Execute
        start_time = time.time()
        result = operation_func(count)
        end_time = time.time()

        # Measure
        gc.collect()
        end_memory = self.get_memory_usage()

        duration = end_time - start_time
        rate = count / duration if duration > 0 else float('inf')
        memory_growth = end_memory - start_memory

        return PerformanceResult(
            operation=operation_name,
            rate_per_second=rate,
            memory_growth_mb=memory_growth,
            duration_seconds=duration,
            items_processed=count,
            passed=True,  # Will be updated based on baselines
            details={'result': result}
        )

    def test_config_creation_performance(self):
        """Test configuration creation performance regression."""
        print("\n⚡ Testing config creation performance...")

        def create_configs(count: int) -> List:
            configs = []
            for i in range(count):
                config = nexus_nitro_llm.PyConfig(
                    lightllm_url=f"http://perf-test-{i % 100}.local:8000",
                    model_id=f"perf-model-{i % 20}",
                    port=3000 + (i % 1000)
                )
                configs.append(config)
            return configs

        result = self.measure_performance("Config Creation", create_configs, 5000)

        print(f"  Rate: {result.rate_per_second:,.0f} configs/second")
        print(f"  Memory growth: {result.memory_growth_mb:.1f}MB")
        print(f"  Duration: {result.duration_seconds:.3f}s")

        # Performance assertions
        assert result.rate_per_second >= self.baselines.config_creation_per_sec * 0.8, \
            f"Config creation too slow: {result.rate_per_second:.0f} < {self.baselines.config_creation_per_sec * 0.8:.0f}/sec"

        assert result.memory_growth_mb < self.baselines.max_memory_growth_mb, \
            f"Excessive memory usage: {result.memory_growth_mb:.1f}MB"

    def test_message_creation_performance(self):
        """Test message creation performance regression."""
        print("\n📝 Testing message creation performance...")

        def create_messages(count: int) -> List:
            messages = []
            roles = ["system", "user", "assistant"]
            for i in range(count):
                role = roles[i % len(roles)]
                content = f"Performance test message {i} with some content to make it realistic."
                msg = nexus_nitro_llm.create_message(role, content)
                messages.append(msg)
            return messages

        result = self.measure_performance("Message Creation", create_messages, 10000)

        print(f"  Rate: {result.rate_per_second:,.0f} messages/second")
        print(f"  Memory growth: {result.memory_growth_mb:.1f}MB")
        print(f"  Duration: {result.duration_seconds:.3f}s")

        # Performance assertions
        assert result.rate_per_second >= self.baselines.message_creation_per_sec * 0.8, \
            f"Message creation too slow: {result.rate_per_second:.0f} < {self.baselines.message_creation_per_sec * 0.8:.0f}/sec"

        assert result.memory_growth_mb < self.baselines.max_memory_growth_mb, \
            f"Excessive memory usage: {result.memory_growth_mb:.1f}MB"

    def test_client_creation_performance(self):
        """Test client creation performance regression."""
        print("\n🔧 Testing client creation performance...")

        def create_clients(count: int) -> List:
            clients = []
            for i in range(count):
                config = nexus_nitro_llm.PyConfig(
                    lightllm_url=f"http://client-perf-{i}.local:8000",
                    model_id=f"client-model-{i}"
                )
                client = nexus_nitro_llm.PyLightLLMClient(config)
                clients.append(client)
            return clients

        result = self.measure_performance("Client Creation", create_clients, 100)

        print(f"  Rate: {result.rate_per_second:.0f} clients/second")
        print(f"  Memory growth: {result.memory_growth_mb:.1f}MB")
        print(f"  Duration: {result.duration_seconds:.3f}s")

        # Performance assertions
        assert result.rate_per_second >= self.baselines.client_creation_per_sec * 0.8, \
            f"Client creation too slow: {result.rate_per_second:.0f} < {self.baselines.client_creation_per_sec * 0.8:.0f}/sec"

        assert result.memory_growth_mb < self.baselines.max_memory_growth_mb, \
            f"Excessive memory usage: {result.memory_growth_mb:.1f}MB"

    def test_stats_retrieval_performance(self):
        """Test get_stats() performance regression."""
        print("\n📊 Testing stats retrieval performance...")

        # Pre-create client
        config = nexus_nitro_llm.PyConfig(
            lightllm_url="http://stats-test.local:8000",
            model_id="stats-model"
        )
        client = nexus_nitro_llm.PyLightLLMClient(config)

        def get_stats_repeatedly(count: int) -> List:
            stats_list = []
            for i in range(count):
                stats = client.get_stats()
                stats_list.append(stats)
            return stats_list

        result = self.measure_performance("Stats Retrieval", get_stats_repeatedly, 2000)

        print(f"  Rate: {result.rate_per_second:,.0f} stats/second")
        print(f"  Memory growth: {result.memory_growth_mb:.1f}MB")
        print(f"  Duration: {result.duration_seconds:.3f}s")

        # Performance assertions
        assert result.rate_per_second >= self.baselines.stats_retrieval_per_sec * 0.8, \
            f"Stats retrieval too slow: {result.rate_per_second:.0f} < {self.baselines.stats_retrieval_per_sec * 0.8:.0f}/sec"

        # Stats retrieval should have minimal memory growth
        assert result.memory_growth_mb < 10.0, \
            f"Stats retrieval memory usage too high: {result.memory_growth_mb:.1f}MB"

    def test_mixed_operations_performance(self):
        """Test performance of mixed operations under realistic load."""
        print("\n🎯 Testing mixed operations performance...")

        def mixed_operations(count: int) -> Dict:
            results = {
                'configs': 0,
                'clients': 0,
                'messages': 0,
                'stats_calls': 0
            }

            configs = []
            clients = []

            for i in range(count):
                # Create config (20% of operations)
                if i % 5 == 0:
                    config = nexus_nitro_llm.PyConfig(
                        lightllm_url=f"http://mixed-{i}.local:8000",
                        model_id=f"mixed-{i}"
                    )
                    configs.append(config)
                    results['configs'] += 1

                    # Create client from config (10% of operations)
                    if i % 10 == 0:
                        client = nexus_nitro_llm.PyLightLLMClient(config)
                        clients.append(client)
                        results['clients'] += 1

                # Create messages (50% of operations)
                if i % 2 == 0:
                    msg = nexus_nitro_llm.create_message(
                        "user" if i % 4 == 0 else "assistant",
                        f"Mixed operation test message {i}"
                    )
                    results['messages'] += 1

                # Get stats from existing clients (remaining operations)
                if clients and i % 3 == 0:
                    client = clients[i % len(clients)]
                    stats = client.get_stats()
                    results['stats_calls'] += 1

            return results

        result = self.measure_performance("Mixed Operations", mixed_operations, 1000)
        mixed_results = result.details['result']

        print(f"  Overall rate: {result.rate_per_second:.0f} operations/second")
        print(f"  Memory growth: {result.memory_growth_mb:.1f}MB")
        print(f"  Duration: {result.duration_seconds:.3f}s")
        print(f"  Breakdown:")
        print(f"    Configs created: {mixed_results['configs']}")
        print(f"    Clients created: {mixed_results['clients']}")
        print(f"    Messages created: {mixed_results['messages']}")
        print(f"    Stats calls: {mixed_results['stats_calls']}")

        # Should handle mixed load efficiently
        assert result.rate_per_second >= 500, \
            f"Mixed operations too slow: {result.rate_per_second:.0f} < 500/sec"

        assert result.memory_growth_mb < self.baselines.max_memory_growth_mb, \
            f"Excessive memory usage in mixed operations: {result.memory_growth_mb:.1f}MB"

    def test_memory_efficiency_over_time(self):
        """Test that memory usage remains stable over extended operations."""
        print("\n📈 Testing memory efficiency over time...")

        config = nexus_nitro_llm.PyConfig(
            lightllm_url="http://memory-test.local:8000",
            model_id="memory-model"
        )

        memory_samples = []
        operations_count = 0

        # Sample memory every 100 operations for 1000 total operations
        for cycle in range(10):
            cycle_start_memory = self.get_memory_usage()

            # Perform 100 operations in this cycle
            for i in range(100):
                # Mix of operations
                if i % 10 == 0:
                    # Create and immediately use client
                    client = nexus_nitro_llm.PyLightLLMClient(config)
                    stats = client.get_stats()
                    operations_count += 2
                else:
                    # Create message
                    msg = nexus_nitro_llm.create_message("user", f"Memory test {cycle}-{i}")
                    operations_count += 1

            # Force cleanup and sample memory
            gc.collect()
            cycle_end_memory = self.get_memory_usage()
            memory_samples.append({
                'cycle': cycle,
                'operations': operations_count,
                'memory_mb': cycle_end_memory,
                'cycle_growth': cycle_end_memory - cycle_start_memory
            })

        # Analyze memory trend
        memory_values = [sample['memory_mb'] for sample in memory_samples]
        memory_growth_per_cycle = [sample['cycle_growth'] for sample in memory_samples]

        initial_memory = memory_samples[0]['memory_mb']
        final_memory = memory_samples[-1]['memory_mb']
        total_growth = final_memory - initial_memory

        avg_cycle_growth = statistics.mean(memory_growth_per_cycle)
        max_cycle_growth = max(memory_growth_per_cycle)

        print(f"  Total operations: {operations_count:,}")
        print(f"  Initial memory: {initial_memory:.1f}MB")
        print(f"  Final memory: {final_memory:.1f}MB")
        print(f"  Total growth: {total_growth:.1f}MB")
        print(f"  Average cycle growth: {avg_cycle_growth:.2f}MB")
        print(f"  Max cycle growth: {max_cycle_growth:.2f}MB")

        # Memory efficiency assertions
        assert total_growth < 50, f"Excessive total memory growth: {total_growth:.1f}MB"
        assert avg_cycle_growth < 5.0, f"High average cycle growth: {avg_cycle_growth:.2f}MB"
        assert max_cycle_growth < 15.0, f"High peak cycle growth: {max_cycle_growth:.2f}MB"

    def test_large_batch_processing_performance(self):
        """Test performance with large batches of data."""
        print("\n🏋️ Testing large batch processing performance...")

        def process_large_batch(batch_size: int) -> Dict:
            # Create a large batch of configurations
            configs = []
            for i in range(batch_size):
                config = nexus_nitro_llm.PyConfig(
                    lightllm_url=f"http://batch-{i}.local:8000",
                    model_id=f"batch-model-{i % 50}"  # Reduce variety for realism
                )
                configs.append(config)

            # Create clients from configs
            clients = []
            for i, config in enumerate(configs[:min(50, batch_size)]):  # Limit clients
                client = nexus_nitro_llm.PyLightLLMClient(config)
                clients.append(client)

            # Create many messages
            messages = []
            for i in range(batch_size * 2):  # 2x messages as configs
                msg = nexus_nitro_llm.create_message(
                    "user" if i % 2 == 0 else "assistant",
                    f"Large batch processing message {i} with realistic content length."
                )
                messages.append(msg)

            # Get stats from all clients
            stats_results = []
            for client in clients:
                stats = client.get_stats()
                stats_results.append(stats)

            return {
                'configs': len(configs),
                'clients': len(clients),
                'messages': len(messages),
                'stats_calls': len(stats_results)
            }

        result = self.measure_performance("Large Batch Processing", process_large_batch, 500)
        batch_results = result.details['result']

        total_objects = sum(batch_results.values())

        print(f"  Batch processing rate: {result.rate_per_second:.0f} batches/second")
        print(f"  Total objects processed: {total_objects:,}")
        print(f"  Objects per second: {total_objects / result.duration_seconds:,.0f}")
        print(f"  Memory growth: {result.memory_growth_mb:.1f}MB")
        print(f"  Memory per object: {result.memory_growth_mb / total_objects * 1024:.1f}KB")

        # Large batch performance assertions
        objects_per_second = total_objects / result.duration_seconds
        assert objects_per_second >= 1000, \
            f"Large batch processing too slow: {objects_per_second:.0f} < 1000 objects/sec"

        memory_per_object = result.memory_growth_mb / total_objects * 1024  # KB per object
        assert memory_per_object < 1.0, \
            f"High memory per object: {memory_per_object:.2f}KB"

    def test_performance_consistency(self):
        """Test that performance is consistent across multiple runs."""
        print("\n🎯 Testing performance consistency...")

        def single_run_operation(count: int) -> int:
            """Single run of mixed operations for consistency testing."""
            operations = 0

            # Create some configs
            configs = []
            for i in range(count // 10):
                config = nexus_nitro_llm.PyConfig(
                    lightllm_url=f"http://consistency-{i}.local:8000",
                    model_id=f"consistency-{i}"
                )
                configs.append(config)
                operations += 1

            # Create messages
            for i in range(count // 2):
                msg = nexus_nitro_llm.create_message("user", f"Consistency test {i}")
                operations += 1

            # Create some clients and get stats
            for config in configs[:min(5, len(configs))]:
                client = nexus_nitro_llm.PyLightLLMClient(config)
                stats = client.get_stats()
                operations += 2

            return operations

        # Run multiple times and measure consistency
        run_results = []
        for run in range(10):
            result = self.measure_performance(f"Consistency Run {run+1}", single_run_operation, 100)
            run_results.append(result.rate_per_second)
            print(f"  Run {run+1}: {result.rate_per_second:.0f} ops/sec")

        # Analyze consistency
        avg_rate = statistics.mean(run_results)
        std_dev = statistics.stdev(run_results)
        min_rate = min(run_results)
        max_rate = max(run_results)
        coefficient_of_variation = (std_dev / avg_rate) * 100

        print(f"  Average rate: {avg_rate:.0f} ops/sec")
        print(f"  Standard deviation: {std_dev:.1f}")
        print(f"  Min rate: {min_rate:.0f} ops/sec")
        print(f"  Max rate: {max_rate:.0f} ops/sec")
        print(f"  Coefficient of variation: {coefficient_of_variation:.1f}%")

        # Performance should be consistent (low coefficient of variation)
        assert coefficient_of_variation < 20.0, \
            f"Performance too inconsistent: {coefficient_of_variation:.1f}% variation"

        # No run should be dramatically slower
        assert min_rate >= avg_rate * 0.7, \
            f"Inconsistent performance: slowest run {min_rate:.0f} much slower than average {avg_rate:.0f}"


if __name__ == "__main__":
    # Run performance regression tests when executed directly
    pytest.main([__file__, "-v", "-s"])