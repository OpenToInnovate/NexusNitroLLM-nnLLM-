#!/usr/bin/env python3
"""
# Minimal Python Smoke Test

Lean, fast smoke test using only Python standard library.
"""

import json
import socket
import ssl
import time
import urllib.parse
import urllib.request
import os
from http.client import HTTPConnection, HTTPSConnection


def make_request(url, method='GET', body=None, headers=None, timeout=5):
    """Make HTTP request using standard library only"""
    parsed = urllib.parse.urlparse(url)
    
    if parsed.scheme == 'https':
        conn = HTTPSConnection(parsed.netloc, timeout=timeout)
    else:
        conn = HTTPConnection(parsed.netloc, timeout=timeout)
    
    try:
        if headers is None:
            headers = {}
        
        conn.request(method, parsed.path, body, headers)
        response = conn.getresponse()
        
        data = response.read().decode('utf-8')
        
        return {
            'status': response.status,
            'headers': dict(response.getheaders()),
            'data': data
        }
    finally:
        conn.close()


def smoke_test():
    base_url = os.getenv('BASE_URL', 'http://localhost:3000')
    
    print(f'🚀 Running minimal Python smoke test against {base_url}')
    
    try:
        # Test 1: Health check
        print('🧪 Testing health endpoint...')
        start1 = time.time()
        health_response = make_request(f'{base_url}/health')
        elapsed1 = (time.time() - start1) * 1000
        
        if health_response['status'] == 200:
            print(f'✅ Health check passed in {elapsed1:.0f}ms')
        else:
            print(f'❌ Health check failed: {health_response["status"]}')
            return
        
        # Test 2: Chat completion
        print('🧪 Testing chat completion...')
        start2 = time.time()
        body = json.dumps({
            'model': 'test-model',
            'messages': [{'role': 'user', 'content': 'Hello'}],
            'max_tokens': 10
        })
        
        chat_response = make_request(f'{base_url}/v1/chat/completions', 
                                   method='POST',
                                   body=body,
                                   headers={'Content-Type': 'application/json'})
        
        elapsed2 = (time.time() - start2) * 1000
        
        if chat_response['status'] == 200:
            data = json.loads(chat_response['data'])
            print(f'✅ Chat completion passed in {elapsed2:.0f}ms')
            print(f'   Response ID: {data.get("id", "unknown")}')
        else:
            print(f'❌ Chat completion failed: {chat_response["status"]}')
        
        # Test 3: Cancellation (quick timeout)
        print('🧪 Testing cancellation...')
        start3 = time.time()
        
        try:
            make_request(f'{base_url}/v1/chat/completions',
                        method='POST',
                        body=body,
                        headers={'Content-Type': 'application/json'},
                        timeout=0.1)
            print('⚠️  Expected timeout, but got response')
        except (socket.timeout, ConnectionError, OSError):
            print(f'✅ Timeout test passed in {(time.time() - start3) * 1000:.0f}ms')
        except Exception as e:
            print(f'❌ Unexpected error: {e}')
        
        print('🎉 Python smoke test completed!')
        
    except Exception as error:
        print(f'❌ Smoke test failed: {error}')
        exit(1)


if __name__ == '__main__':
    smoke_test()

