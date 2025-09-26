#!/usr/bin/env python3
"""
Test script to verify Python true streaming implementation
"""
import asyncio
import nexus_nitro_llm

async def test_python_streaming():
    """Test the Python async streaming functionality"""
    print("Testing Python async streaming implementation...")

    # Create async streaming client
    try:
        config = nexus_nitro_llm.create_config(
            backend_url="direct",
            model_id="llama-2-7b-chat"
        )
        client = nexus_nitro_llm.PyAsyncStreamingClient(config)

        print("✓ Async streaming client created successfully")

        # Test streaming chat
        messages = [
            nexus_nitro_llm.create_message("user", "Hello, tell me about programming")
        ]

        print("Sending streaming request...")
        stream = await client.stream_chat_completions_async(messages)

        print("Receiving streaming chunks:")
        chunk_count = 0
        async for chunk in stream:
            chunk_count += 1
            if chunk and 'choices' in chunk and len(chunk['choices']) > 0:
                delta = chunk['choices'][0].get('delta', {})
                content = delta.get('content', '')
                if content:
                    print(f"Chunk {chunk_count}: '{content.strip()}'")

                # Check if this is the final chunk
                finish_reason = chunk['choices'][0].get('finish_reason')
                if finish_reason == 'stop':
                    print(f"✓ Stream completed with finish_reason: {finish_reason}")
                    break

        print(f"✓ Received {chunk_count} streaming chunks")
        print("✓ Python streaming test passed!")

    except Exception as e:
        print(f"✗ Python streaming test failed: {e}")
        return False

    return True

if __name__ == "__main__":
    success = asyncio.run(test_python_streaming())
    if success:
        print("\n🎉 All Python streaming tests passed!")
    else:
        print("\n❌ Python streaming tests failed!")
        exit(1)