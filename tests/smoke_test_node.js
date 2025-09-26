#!/usr/bin/env node
/**
 * # Minimal Node.js Smoke Test
 * 
 * Lean, fast smoke test with zero dependencies beyond Node.js built-ins.
 */

const http = require('http');
const https = require('https');
const { URL } = require('url');

async function makeRequest(url, options = {}) {
    return new Promise((resolve, reject) => {
        const parsedUrl = new URL(url);
        const isHttps = parsedUrl.protocol === 'https:';
        const client = isHttps ? https : http;
        
        const requestOptions = {
            hostname: parsedUrl.hostname,
            port: parsedUrl.port || (isHttps ? 443 : 80),
            path: parsedUrl.pathname + parsedUrl.search,
            method: options.method || 'GET',
            headers: options.headers || {},
            timeout: options.timeout || 5000
        };
        
        const req = client.request(requestOptions, (res) => {
            let data = '';
            res.on('data', chunk => data += chunk);
            res.on('end', () => resolve({
                status: res.statusCode,
                headers: res.headers,
                data: data
            }));
        });
        
        req.on('error', reject);
        req.on('timeout', () => {
            req.destroy();
            reject(new Error('Request timeout'));
        });
        
        if (options.body) {
            req.write(options.body);
        }
        
        req.end();
    });
}

async function smokeTest() {
    const baseUrl = process.env.BASE_URL || 'http://localhost:3000';
    
    console.log(`🚀 Running minimal Node.js smoke test against ${baseUrl}`);
    
    try {
        // Test 1: Health check
        console.log('🧪 Testing health endpoint...');
        const start1 = Date.now();
        const healthResponse = await makeRequest(`${baseUrl}/health`);
        const elapsed1 = Date.now() - start1;
        
        if (healthResponse.status === 200) {
            console.log(`✅ Health check passed in ${elapsed1}ms`);
        } else {
            console.log(`❌ Health check failed: ${healthResponse.status}`);
            return;
        }
        
        // Test 2: Chat completion
        console.log('🧪 Testing chat completion...');
        const start2 = Date.now();
        const body = JSON.stringify({
            model: 'test-model',
            messages: [{ role: 'user', content: 'Hello' }],
            max_tokens: 10
        });
        
        const chatResponse = await makeRequest(`${baseUrl}/v1/chat/completions`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Content-Length': Buffer.byteLength(body)
            },
            body: body
        });
        
        const elapsed2 = Date.now() - start2;
        
        if (chatResponse.status === 200) {
            const data = JSON.parse(chatResponse.data);
            console.log(`✅ Chat completion passed in ${elapsed2}ms`);
            console.log(`   Response ID: ${data.id || 'unknown'}`);
        } else {
            console.log(`❌ Chat completion failed: ${chatResponse.status}`);
        }
        
        // Test 3: Cancellation (quick timeout)
        console.log('🧪 Testing cancellation...');
        const start3 = Date.now();
        
        try {
            await makeRequest(`${baseUrl}/v1/chat/completions`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Content-Length': Buffer.byteLength(body)
                },
                body: body,
                timeout: 100
            });
            console.log('⚠️  Expected timeout, but got response');
        } catch (error) {
            if (error.message === 'Request timeout') {
                console.log(`✅ Timeout test passed in ${Date.now() - start3}ms`);
            } else {
                console.log(`❌ Unexpected error: ${error.message}`);
            }
        }
        
        console.log('🎉 Node.js smoke test completed!');
        
    } catch (error) {
        console.error(`❌ Smoke test failed: ${error.message}`);
        process.exit(1);
    }
}

if (require.main === module) {
    smokeTest();
}

module.exports = { smokeTest };

