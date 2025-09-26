"""
Type stubs for NexusNitroLLM Python bindings.

This file provides comprehensive type annotations for the universal LLM proxy bindings,
enabling better IDE support, type checking, and developer experience.
"""

from typing import Dict, List, Optional, Any, Union
from typing_extensions import Literal

# Exception types
class NexusNitroLLMError(Exception):
    """Base exception for universal LLM operations."""
    def __init__(self, message: str, error_type: str) -> None: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

class ConnectionError(NexusNitroLLMError):
    """Exception for network-related issues."""
    def __init__(self, message: str, url: str) -> None: ...
    def __str__(self) -> str: ...

class ConfigurationError(NexusNitroLLMError):
    """Exception for configuration-related issues."""
    def __init__(self, message: str, field: str) -> None: ...
    def __str__(self) -> str: ...

# Core classes
class PyConfig:
    """Configuration for universal LLM proxy."""
    
    def __init__(
        self,
        backend_url: Optional[str] = None,
        backend_type: Optional[str] = None,
        model_id: Optional[str] = None,
        port: Optional[int] = None,
        token: Optional[str] = None,
        timeout: Optional[int] = None
    ) -> None: ...
    
    @property
    def backend_url(self) -> str: ...
    
    @property
    def model_id(self) -> str: ...
    
    def set_backend_url(self, url: str) -> None: ...
    def set_model_id(self, model_id: str) -> None: ...
    def set_token(self, token: str) -> None: ...
    def set_connection_pooling(self, enabled: bool) -> None: ...

class PyMessage:
    """Message for chat completions."""
    
    def __init__(self, role: str, content: str) -> None: ...
    
    @property
    def role(self) -> str: ...
    
    @property
    def content(self) -> str: ...
    
    def set_content(self, content: str) -> None: ...

class PyNexusNitroLLMClient:
    """High-performance LightLLM client."""
    
    def __init__(self, config: PyConfig) -> None: ...
    
    def chat_completions(
        self,
        messages: List[PyMessage],
        model: Optional[str] = None,
        max_tokens: Optional[int] = None,
        temperature: Optional[float] = None,
        stream: bool = False
    ) -> Dict[str, Any]: ...
    
    def get_stats(self) -> Dict[str, Any]: ...
    def get_performance_metrics(self) -> Dict[str, Any]: ...
    def test_connection(self) -> bool: ...

class PyAsyncNexusNitroLLMClient:
    """Async-compatible LightLLM client for asyncio applications."""
    
    def __init__(self, config: PyConfig) -> None: ...
    
    def chat_completions_async(
        self,
        messages: List[PyMessage],
        model: Optional[str] = None,
        max_tokens: Optional[int] = None,
        temperature: Optional[float] = None,
        stream: bool = False
    ) -> Any: ...  # Returns a coroutine
    
    def get_stats(self) -> Dict[str, Any]: ...
    def test_connection_async(self) -> Any: ...  # Returns a coroutine

class PyStreamingClient:
    """Streaming client for real-time responses."""
    
    def __init__(self, config: PyConfig) -> None: ...
    
    def stream_chat_completions(
        self,
        messages: List[PyMessage],
        model: Optional[str] = None,
        max_tokens: Optional[int] = None,
        temperature: Optional[float] = None
    ) -> Dict[str, Any]: ...

class PyAsyncStreamingClient:
    """Async streaming client for real-time responses."""
    
    def __init__(self, config: PyConfig) -> None: ...
    
    def stream_chat_completions_async(
        self,
        messages: List[PyMessage],
        model: Optional[str] = None,
        max_tokens: Optional[int] = None,
        temperature: Optional[float] = None
    ) -> Any: ...  # Returns a coroutine

# Module-level functions
def create_client(
    backend_url: str,
    backend_type: Optional[str] = None,
    model_id: Optional[str] = None,
    token: Optional[str] = None,
    timeout: Optional[int] = None
) -> PyNexusNitroLLMClient: ...

def create_async_client(
    backend_url: str,
    backend_type: Optional[str] = None,
    model_id: Optional[str] = None,
    token: Optional[str] = None,
    timeout: Optional[int] = None
) -> PyAsyncNexusNitroLLMClient: ...

def create_message(role: str, content: str) -> PyMessage: ...

def create_config(
    backend_url: Optional[str] = None,
    backend_type: Optional[str] = None,
    model_id: Optional[str] = None,
    port: Optional[int] = None,
    token: Optional[str] = None,
    timeout: Optional[int] = None
) -> PyConfig: ...

# Module metadata
__version__: str
__author__: str
__doc__: str
