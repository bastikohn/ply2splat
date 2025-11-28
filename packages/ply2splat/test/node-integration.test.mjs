/**
 * Integration tests for the ply2splat public API in Node.js
 * 
 * This verifies that the package's entry point (dist/index.js) works correctly
 * in a Node.js environment, specifically handling WASM loading via 'fs'
 * instead of 'fetch' (which fails in Node.js for local files).
 */

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { init, convert, getBackend } from '../dist/index.js';

// Minimal PLY data for testing
const MINIMAL_PLY_DATA = `ply
format ascii 1.0
element vertex 1
property float x
property float y
property float z
property float f_dc_0
property float f_dc_1
property float f_dc_2
property float opacity
property float scale_0
property float scale_1
property float scale_2
property float rot_0
property float rot_1
property float rot_2
property float rot_3
end_header
1.0 2.0 3.0 0.5 0.5 0.5 0.0 0.1 0.1 0.1 1.0 0.0 0.0 0.0
`;

describe('Node.js Integration', () => {
  test('init() should successfully load WASM without fetch error', async () => {
    // This is the critical test for the "fetch failed" fix.
    // If the Node.js detection in src/index.ts fails, this will try to use fetch and throw.
    await assert.doesNotReject(async () => {
      await init();
    }, 'init() threw an error during initialization');
  });

  test('getBackend() should return "wasm" (or "native" if available)', async () => {
    await init();
    const backend = getBackend();
    assert.ok(backend === 'wasm' || backend === 'native', `Backend should be wasm or native, got ${backend}`);
    // In this environment, we expect WASM, but being flexible is good
    // console.log(`Using backend: ${backend}`);
  });

  test('convert() should process data via the public API', async () => {
    await init();
    const plyData = new TextEncoder().encode(MINIMAL_PLY_DATA);
    
    const result = convert(plyData);
    
    assert.strictEqual(result.count, 1, 'Should have 1 splat');
    assert.strictEqual(result.data.length, 32, 'Data length should be 32 bytes');
    
    // Verify method existence on result object
    assert.strictEqual(typeof result.toSplats, 'function');
    assert.strictEqual(typeof result.free, 'function');
    
    // Test result parsing
    const splats = result.toSplats();
    assert.strictEqual(splats.length, 1);
    assert.strictEqual(splats[0].position.length, 3);
  });
});
