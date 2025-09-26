# NexusNitroLLM Node.js Bindings

High-performance Node.js bindings for NexusNitroLLM (nnLLM) - Universal LLM integration library.

## Features

- **Universal LLM Support**: LightLLM, vLLM, Azure OpenAI, AWS Bedrock, OpenAI, and custom endpoints
- **Direct Mode**: Bypass HTTP for maximum performance when using embedded models
- **Zero-Copy Operations**: Minimal memory allocations and data copying
- **Connection Pooling**: Reuse HTTP connections for better performance
- **TypeScript Support**: Full type definitions included
- **Async/Await**: Native Promise support
- **High Performance**: Built with Rust for maximum speed

## Installation

```bash
npm install nexus-nitro-llm
```

## Quick Start

### Direct Mode (Maximum Performance)

```javascript
const { create_direct_client, NodeMessage } = require('nexus-nitro-llm');

// Create a direct mode client (no HTTP overhead)
const client = create_direct_client('my-model');

// Send a chat completion request
const response = await client.chat_completions({
  messages: [new NodeMessage('user', 'What is the capital of France?')],
  max_tokens: 50
});

console.log(response.choices[0].message.content);
```

### HTTP Mode (Remote Backend)

```javascript
const { create_http_client, NodeMessage } = require('nexus-nitro-llm');

// Create an HTTP mode client
const client = create_http_client('http://localhost:8000', 'my-model');

// Send a chat completion request
const response = await client.chat_completions({
  messages: [new NodeMessage('user', 'What is the capital of Canada?')],
  max_tokens: 50
});

console.log(response.choices[0].message.content);
```

### TypeScript Usage

```typescript
import { create_direct_client, create_http_client, NodeMessage, NodeLightLLMClient } from 'nexus-nitro-llm';

// Direct mode with type safety
const directClient: NodeLightLLMClient = create_direct_client('my-model');

// HTTP mode with type safety
const httpClient: NodeLightLLMClient = create_http_client('http://localhost:8000', 'my-model');

// Type-safe chat completion
const response = await directClient.chat_completions({
  messages: [new NodeMessage('user', 'Hello, world!')],
  max_tokens: 100,
  temperature: 0.7
});
```

## API Reference

### Client Creation

#### `create_direct_client(modelId?, token?)`
Creates a client for direct mode (no HTTP overhead).

- `modelId` (optional): Model identifier
- `token` (optional): Authentication token

#### `create_http_client(url, modelId?, token?)`
Creates a client for HTTP mode (remote backend).

- `url`: Backend URL (e.g., `http://localhost:8000`)
- `modelId` (optional): Model identifier
- `token` (optional): Authentication token

### Chat Completions

#### `client.chat_completions(options)`
Send a chat completion request.

**Options:**
- `messages`: Array of `NodeMessage` objects
- `max_tokens` (optional): Maximum tokens to generate
- `temperature` (optional): Sampling temperature (0.0-2.0)
- `top_p` (optional): Nucleus sampling parameter
- `stream` (optional): Enable streaming (boolean)

**Returns:** Promise resolving to chat completion response

### Message Creation

#### `new NodeMessage(role, content)`
Create a chat message.

- `role`: Message role (`'user'`, `'assistant'`, `'system'`)
- `content`: Message content

### Client Statistics

#### `client.get_stats()`
Get client statistics and configuration.

**Returns:** Object with:
- `adapter_type`: Adapter type (`'direct'`, `'lightllm'`, `'openai'`, etc.)
- `backend_url`: Backend URL
- `model_id`: Model identifier
- `is_direct_mode`: Whether in direct mode
- `performance_mode`: Performance mode description
- `total_requests`: Total requests made
- `total_errors`: Total errors encountered

## Supported Backends

### LightLLM
```javascript
const client = create_http_client('http://localhost:8000', 'llama');
```

### vLLM
```javascript
const client = create_http_client('http://localhost:8000', 'llama');
```

### Azure OpenAI
```javascript
const client = create_http_client('https://your-resource.openai.azure.com', 'gpt-4');
```

### AWS Bedrock
```javascript
const client = create_http_client('https://bedrock.us-east-1.amazonaws.com', 'anthropic.claude-3-sonnet');
```

### OpenAI
```javascript
const client = create_http_client('https://api.openai.com/v1', 'gpt-4');
```

### Custom Endpoints
```javascript
const client = create_http_client('https://your-custom-endpoint.com', 'your-model');
```

## Performance Tips

1. **Use Direct Mode**: For embedded models, use `create_direct_client()` to bypass HTTP overhead
2. **Connection Pooling**: HTTP clients automatically reuse connections
3. **Batch Requests**: Send multiple requests concurrently for better throughput
4. **Reuse Clients**: Create clients once and reuse them

## Examples

See the `examples/` directory for:
- Basic usage examples
- Direct mode usage
- Performance comparisons
- TypeScript examples

## Development

### Building from Source

```bash
# Install dependencies
npm install

# Build the native module
npm run build

# Run tests
npm test

# Run benchmarks
npm run bench
```

### Requirements

- Node.js >= 16
- Rust toolchain (for building from source)

## License

MIT OR Apache-2.0

## Contributing

Contributions are welcome! Please see our [contributing guidelines](https://github.com/nexusnitro/nexus-nitro-llm/blob/main/CONTRIBUTING.md).

## Support

- [Documentation](https://docs.rs/nexus_nitro_llm)
- [Issues](https://github.com/nexusnitro/nexus-nitro-llm/issues)
- [Discussions](https://github.com/nexusnitro/nexus-nitro-llm/discussions)
