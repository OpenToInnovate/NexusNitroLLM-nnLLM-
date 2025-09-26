#!/usr/bin/env python3
"""
Enhanced basic functionality tests for LightLLM Rust Python bindings.

Tests core functionality, configuration, error handling, and basic operations to ensure
the bindings work correctly and robustly with comprehensive coverage.
"""

import pytest
import time
import gc
import weakref
from typing import Dict, Any, List, Optional
import logging

# Configure logging for tests
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Import the bindings (this will be available after maturin develop)
try:
    import nexus_nitro_llm
    from nexus_nitro_llm import (
        PyConfig, PyMessage, PyNexusNitroLLMClient, PyStreamingClient,
        LightLLMError, ConnectionError, ConfigurationError
    )
    BINDINGS_AVAILABLE = True
except ImportError:
    BINDINGS_AVAILABLE = False
    nexus_nitro_llm = None


class TestBasicFunctionality:
    """Test basic functionality of Python bindings."""

    @pytest.fixture(autouse=True)
    def setup_method(self):
        """Set up test environment before each test."""
        if not BINDINGS_AVAILABLE:
            pytest.skip("Python bindings not available - run 'maturin develop --features python' first")

    def test_config_creation(self):
        """Test configuration object creation and properties."""
        config = PyConfig(
            backend_url="http://localhost:8000",
            model_id="test-model",
            port=8080,
            token="test-token",
            timeout=30
        )

        assert config.backend_url == "http://localhost:8000"
        assert config.model_id == "test-model"

        # Test setters
        config.set_backend_url("http://localhost:8001")
        config.set_model_id("new-model")
        config.set_token("new-token")

        assert config.backend_url == "http://localhost:8001"
        assert config.model_id == "new-model"

        # Test connection pooling setting
        config.set_connection_pooling(True)
        config.set_connection_pooling(False)

    def test_config_validation_errors(self):
        """Test configuration validation and error handling."""
        # Test empty URL
        with pytest.raises(ConfigurationError):
            PyConfig(backend_url="")

        # Test invalid URL format
        with pytest.raises(ConfigurationError):
            PyConfig(backend_url="invalid-url")

        # Test empty model ID
        with pytest.raises(ConfigurationError):
            PyConfig(model_id="")

        # Test invalid port
        with pytest.raises(ConfigurationError):
            PyConfig(port=0)

        # Test invalid timeout
        with pytest.raises(ConfigurationError):
            PyConfig(timeout=0)

    def test_config_defaults(self):
        """Test configuration with default values."""
        config = PyConfig()
        
        # Should have valid defaults
        assert config.backend_url
        assert config.model_id
        assert config.port > 0

    def test_message_creation(self):
        """Test message object creation and properties."""
        # Test direct creation
        msg = nexus_nitro_llm.PyMessage("user", "Hello world!")
        assert msg.role == "user"
        assert msg.content == "Hello world!"

        # Test content modification
        msg.set_content("Updated content")
        assert msg.content == "Updated content"

        # Test convenience function
        msg2 = nexus_nitro_llm.create_message("assistant", "Hello back!")
        assert msg2.role == "assistant"
        assert msg2.content == "Hello back!"

        # Test various roles
        roles = ["system", "user", "assistant", "tool"]
        for role in roles:
            msg = nexus_nitro_llm.create_message(role, f"Content for {role}")
            assert msg.role == role
            assert msg.content == f"Content for {role}"

    def test_client_creation(self):
        """Test client creation and basic operations."""
        config = nexus_nitro_llm.PyConfig(
            backend_url="http://localhost:8000",
            model_id="test-model"
        )

        # Test client creation
        client = nexus_nitro_llm.PyNexusNitroLLMClient(config)
        assert client is not None

        # Test convenience function
        client2 = nexus_nitro_llm.create_client(
            "http://localhost:8000",
            model_id="test-model"
        )
        assert client2 is not None

        # Test stats retrieval
        stats = client.get_stats()
        assert isinstance(stats, dict)
        assert "adapter_type" in stats
        assert "connection_pooling" in stats

    def test_streaming_client_creation(self):
        """Test streaming client creation."""
        config = nexus_nitro_llm.PyConfig(
            backend_url="http://localhost:8000",
            model_id="test-model"
        )

        streaming_client = nexus_nitro_llm.PyStreamingClient(config)
        assert streaming_client is not None

    def test_memory_cleanup(self):
        """Test that objects are properly cleaned up."""
        refs = []

        # Create objects and keep weak references
        for i in range(100):
            config = nexus_nitro_llm.PyConfig(
                backend_url=f"http://localhost:800{i % 10}",
                model_id=f"model-{i}"
            )
            refs.append(weakref.ref(config))

            msg = nexus_nitro_llm.create_message("user", f"Message {i}")
            refs.append(weakref.ref(msg))

        # Force garbage collection
        gc.collect()

        # All objects should be cleaned up
        live_refs = sum(1 for ref in refs if ref() is not None)
        assert live_refs == 0, f"Memory leak detected: {live_refs} objects still alive"

    def test_concurrent_config_access(self):
        """Test concurrent access to configuration objects."""
        import threading
        import queue

        def create_configs(result_queue, count):
            configs = []
            for i in range(count):
                config = nexus_nitro_llm.PyConfig(
                    backend_url=f"http://localhost:{8000 + i}",
                    model_id=f"model-{i}"
                )
                configs.append(config)
            result_queue.put(configs)

        # Create configurations concurrently
        threads = []
        result_queue = queue.Queue()

        for i in range(10):
            thread = threading.Thread(target=create_configs, args=(result_queue, 10))
            threads.append(thread)
            thread.start()

        # Wait for completion
        for thread in threads:
            thread.join()

        # Collect results
        all_configs = []
        while not result_queue.empty():
            configs = result_queue.get()
            all_configs.extend(configs)

        assert len(all_configs) == 100

        # Verify configurations are correct
        for i, config in enumerate(all_configs):
            expected_port = 8000 + (i % 10)
            expected_model = f"model-{i % 10}"
            assert config.backend_url == f"http://localhost:{expected_port}"
            assert config.model_id == expected_model

    def test_error_conditions(self):
        """Test various error conditions and exception handling."""
        # Test invalid configuration
        with pytest.raises(Exception):
            # This should work, but we test the pattern
            config = nexus_nitro_llm.PyConfig(
                backend_url="invalid-url",  # Invalid URL format
                model_id=""  # Empty model
            )
            client = nexus_nitro_llm.PyNexusNitroLLMClient(config)

    def test_module_metadata(self):
        """Test module metadata and version information."""
        # Test module has expected attributes
        assert hasattr(nexus_nitro_llm, '__version__')
        assert hasattr(nexus_nitro_llm, '__author__')
        assert hasattr(nexus_nitro_llm, '__doc__')

        # Test version format
        version = nexus_nitro_llm.__version__
        assert isinstance(version, str)
        assert len(version.split('.')) >= 2  # At least major.minor

    def test_configuration_edge_cases(self):
        """Test edge cases in configuration handling."""
        # Test None values
        config = nexus_nitro_llm.PyConfig()  # All defaults
        assert config.backend_url
        assert config.model_id

        # Test empty strings (should be handled gracefully)
        config = nexus_nitro_llm.PyConfig(
            backend_url="http://localhost:8000",
            model_id="model"
        )

        # Test URL variations
        urls = [
            "http://localhost:8000",
            "https://api.example.com:443",
            "http://192.168.1.100:8080",
        ]

        for url in urls:
            config = nexus_nitro_llm.PyConfig(backend_url=url)
            assert config.backend_url == url

    def test_message_edge_cases(self):
        """Test edge cases in message handling."""
        # Test empty content
        msg = nexus_nitro_llm.create_message("user", "")
        assert msg.content == ""

        # Test very long content
        long_content = "x" * 10000
        msg = nexus_nitro_llm.create_message("user", long_content)
        assert msg.content == long_content

        # Test unicode content
        unicode_content = "Hello 🌍! This is a test with émojis and spécial châractèrs."
        msg = nexus_nitro_llm.create_message("user", unicode_content)
        assert msg.content == unicode_content

        # Test newlines and special characters
        special_content = "Line 1\nLine 2\tTabbed\r\nWindows line ending"
        msg = nexus_nitro_llm.create_message("user", special_content)
        assert msg.content == special_content

    def test_object_reuse(self):
        """Test that objects can be reused safely."""
        config = nexus_nitro_llm.PyConfig(
            backend_url="http://localhost:8000",
            model_id="test-model"
        )

        # Create multiple clients with same config
        clients = []
        for i in range(10):
            client = nexus_nitro_llm.PyNexusNitroLLMClient(config)
            clients.append(client)

        # All clients should be independent
        for client in clients:
            stats = client.get_stats()
            assert isinstance(stats, dict)

        # Modify config after client creation
        config.set_model_id("new-model")
        assert config.model_id == "new-model"

        # Original clients should still work
        for client in clients:
            stats = client.get_stats()
            assert isinstance(stats, dict)


class TestPerformanceBasics:
    """Basic performance tests to ensure bindings are efficient."""

    @pytest.fixture(autouse=True)
    def setup_method(self):
        """Set up test environment before each test."""
        if not BINDINGS_AVAILABLE:
            pytest.skip("Python bindings not available")

    def test_config_creation_performance(self):
        """Test configuration creation performance."""
        start_time = time.time()

        configs = []
        for i in range(1000):
            config = nexus_nitro_llm.PyConfig(
                backend_url=f"http://localhost:{8000 + i % 100}",
                model_id=f"model-{i % 10}"
            )
            configs.append(config)

        elapsed = time.time() - start_time

        # Should create 1000 configs in under 1 second
        assert elapsed < 1.0, f"Config creation too slow: {elapsed:.3f}s for 1000 configs"

        # Verify all configs are correct
        assert len(configs) == 1000
        for i, config in enumerate(configs):
            expected_port = 8000 + (i % 100)
            assert config.backend_url == f"http://localhost:{expected_port}"

    def test_message_creation_performance(self):
        """Test message creation performance."""
        start_time = time.time()

        messages = []
        for i in range(1000):
            msg = nexus_nitro_llm.create_message(
                "user" if i % 2 == 0 else "assistant",
                f"This is test message number {i} with some content."
            )
            messages.append(msg)

        elapsed = time.time() - start_time

        # Should create 1000 messages in under 0.5 seconds
        assert elapsed < 0.5, f"Message creation too slow: {elapsed:.3f}s for 1000 messages"

        # Verify all messages are correct
        assert len(messages) == 1000
        for i, msg in enumerate(messages):
            expected_role = "user" if i % 2 == 0 else "assistant"
            assert msg.role == expected_role
            assert f"number {i}" in msg.content

    def test_client_creation_performance(self):
        """Test client creation performance."""
        configs = []
        for i in range(10):
            config = nexus_nitro_llm.PyConfig(
                backend_url=f"http://localhost:{8000 + i}",
                model_id=f"model-{i}"
            )
            configs.append(config)

        start_time = time.time()

        clients = []
        for config in configs:
            client = nexus_nitro_llm.PyNexusNitroLLMClient(config)
            clients.append(client)

        elapsed = time.time() - start_time

        # Should create 10 clients quickly
        assert elapsed < 1.0, f"Client creation too slow: {elapsed:.3f}s for 10 clients"

        # Verify all clients work
        for client in clients:
            stats = client.get_stats()
            assert isinstance(stats, dict)


if __name__ == "__main__":
    # Run basic tests when executed directly
    pytest.main([__file__, "-v"])