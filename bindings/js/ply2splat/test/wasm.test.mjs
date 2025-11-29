/**
 * WASM module tests for ply2splat
 * 
 * Run with: node --test test/wasm.test.mjs
 */

import { test, describe, before } from 'node:test';
import assert from 'node:assert';
import fs from 'node:fs';
import path from 'node:path';

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

let wasmModule;

describe('WASM Module', () => {
  before(async () => {
    // Load WASM module
    const wasmPath = path.join(import.meta.dirname, '..', 'wasm', 'ply2splat_wasm_bg.wasm');
    const wasmCode = fs.readFileSync(wasmPath);
    const compiled = await WebAssembly.compile(wasmCode);
    const { initSync, convert, getSplatCount, parseSplatData } = await import('../wasm/ply2splat_wasm.js');
    initSync({ module: compiled });
    
    wasmModule = { convert, getSplatCount, parseSplatData };
  });

  test('WASM module loads successfully', () => {
    assert.ok(wasmModule, 'WASM module should be loaded');
    assert.ok(typeof wasmModule.convert === 'function', 'convert should be a function');
    assert.ok(typeof wasmModule.getSplatCount === 'function', 'getSplatCount should be a function');
    assert.ok(typeof wasmModule.parseSplatData === 'function', 'parseSplatData should be a function');
  });

  test('convert() processes PLY data correctly', () => {
    const plyData = new TextEncoder().encode(MINIMAL_PLY_DATA);
    const result = wasmModule.convert(plyData, true);
    
    assert.strictEqual(result.count, 1, 'Should have 1 splat');
    assert.strictEqual(result.data.length, 32, 'Each splat should be 32 bytes');
  });

  test('getSplatCount() returns correct count', () => {
    const plyData = new TextEncoder().encode(MINIMAL_PLY_DATA);
    const result = wasmModule.convert(plyData, true);
    const count = wasmModule.getSplatCount(result.data);
    
    assert.strictEqual(count, 1, 'Should return 1 for single splat');
  });

  test('parseSplatData() returns typed splat objects', () => {
    const plyData = new TextEncoder().encode(MINIMAL_PLY_DATA);
    const result = wasmModule.convert(plyData, true);
    const splats = wasmModule.parseSplatData(result.data);
    
    assert.strictEqual(splats.length, 1, 'Should have 1 splat');
    
    const splat = splats[0];
    assert.ok(Array.isArray(splat.position), 'position should be an array');
    assert.strictEqual(splat.position.length, 3, 'position should have 3 elements');
    assert.ok(Array.isArray(splat.scale), 'scale should be an array');
    assert.strictEqual(splat.scale.length, 3, 'scale should have 3 elements');
    assert.ok(Array.isArray(splat.color), 'color should be an array');
    assert.strictEqual(splat.color.length, 4, 'color should have 4 elements');
    assert.ok(Array.isArray(splat.rotation), 'rotation should be an array');
    assert.strictEqual(splat.rotation.length, 4, 'rotation should have 4 elements');
  });

  test('convert() without sorting option', () => {
    const plyData = new TextEncoder().encode(MINIMAL_PLY_DATA);
    const result = wasmModule.convert(plyData, false);
    
    assert.strictEqual(result.count, 1, 'Should have 1 splat');
    assert.strictEqual(result.data.length, 32, 'Each splat should be 32 bytes');
  });
});
