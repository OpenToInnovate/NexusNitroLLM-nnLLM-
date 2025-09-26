# Node.js Bindings Issues and Solutions

## Current Status

The NexusNitroLLM Node.js bindings face compilation/linking issues that prevent proper building of the native module.

## Issue Summary

### Problem
- **Compilation**: ✅ Compiles successfully (Rust code validates)
- **Linking**: ❌ Fails with missing NAPI symbols during final linking step
- **Specific Error**: Node.js API functions like `_napi_create_string_utf8`, `_napi_typeof`, etc. are missing

### Error Details
```
ld: symbol(s) not found for architecture arm64
_napi_create_string_utf8, _napi_typeof, _napi_throw, etc.
clang: error: linker command failed with exit code 1
```

## Root Cause Analysis

### 1. Node.js Version Compatibility
- **Current**: Node.js v22.19.0
- **Expected**: Node.js v16-20 (based on package.json engines)
- **Issue**: napi-rs 2.16 may not support Node.js v22

### 2. NAPI Library Versions
- **napi**: 2.16.x
- **napi-derive**: 2.16.x
- **@napi-rs/cli**: 2.18.4

### 3. Environment Issues
- Missing Node.js development headers
- Incorrect linking configuration for macOS ARM64

## Potential Solutions

### Solution 1: Downgrade Node.js Version
```bash
# Install Node.js v20 (recommended)
nvm install 20
nvm use 20

# Rebuild with compatible version
npm install
npx napi build --platform --features nodejs
```

### Solution 2: Update NAPI Dependencies
```toml
# In Cargo.toml
[dependencies]
napi = { version = "2.17", optional = true }
napi-derive = { version = "2.17", optional = true }
```

```json
// In package.json
"devDependencies": {
  "@napi-rs/cli": "^2.19.0"
}
```

### Solution 3: Environment Setup
```bash
# Install Node.js headers explicitly
npm install -g node-gyp
node-gyp install

# Set proper environment variables
export npm_config_target_platform=darwin
export npm_config_target_arch=arm64
```

### Solution 4: Alternative Build Configuration
```bash
# Try with release mode and specific target
npx napi build --platform --release --target aarch64-apple-darwin --features nodejs

# Or without platform-specific optimizations
cargo build --features nodejs --target-dir target-nodejs
```

### Solution 5: Simplify Implementation (Temporary)
- Reduce async/await complexity in nodejs.rs
- Remove Tokio runtime integration temporarily
- Use simpler NAPI patterns

## Testing Status

### Basic Compilation: ✅
```bash
cargo check --features nodejs  # Works
cargo build --features nodejs  # Works until linking
```

### Full Build: ❌
```bash
npx napi build --platform --features nodejs  # Fails at link time
```

### Examples Status: ⏸️ Blocked
- Cannot test examples until basic compilation works
- Examples have minor issues (backend_url vs lightllm_url) that are fixed

## Recommended Action Plan

### Immediate (High Priority)
1. **Downgrade Node.js to v20** - Most likely to resolve the issue
2. **Update napi dependencies** to latest versions
3. **Test with minimal binding** first

### Short Term (Medium Priority)
1. **Environment setup** - Ensure proper Node.js headers
2. **Alternative build approaches** - Try different compilation flags
3. **Simplify nodejs.rs** - Remove complex async patterns temporarily

### Long Term (Low Priority)
1. **CI/CD setup** - Test across multiple Node.js versions
2. **Documentation** - Proper setup instructions for users
3. **Version matrix** - Document supported Node.js/napi combinations

## Current Workarounds

### For Development
- Python bindings work perfectly (use those for testing core functionality)
- HTTP API mode works (start server and use HTTP client)
- Rust library functions directly (for testing core logic)

### For Users
- Recommend Python bindings for now
- Provide HTTP server mode as alternative
- Document Node.js requirements clearly

## Files Involved

### Configuration
- `Cargo.toml` - Rust dependencies and features
- `package.json` - Node.js dependencies and engine requirements
- `napi.toml` - NAPI-specific build configuration

### Source Code
- `src/nodejs.rs` - Main Node.js bindings implementation
- `nodejs/examples/basic_usage.js` - Basic usage example (minor fixes applied)

### Generated Files
- `index.js` - Auto-generated NAPI loader
- `index.d.ts` - TypeScript definitions
- `*.node` - Native binary (fails to build with nodejs feature)

## Notes
- The Rust library core is solid and well-tested
- Issue is specifically with Node.js FFI layer, not core functionality
- Python bindings demonstrate the core library works perfectly
- This is a common issue with NAPI/Node.js version mismatches