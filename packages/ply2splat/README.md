# ply2splat (npm)

Convert Gaussian Splatting PLY files to SPLAT format with **Dual-Backend Support** (Native & WASM).

This package automatically selects the best available backend for your environment:

1.  **Native Node.js Bindings**: Used automatically when running in Node.js if the optional `ply2splat-native` package is installed and supported on your platform. This offers **maximum performance** using multi-threading (rayon) and AVX/SIMD instructions.
2.  **WebAssembly (WASM)**: Used in browsers or as a fallback in Node.js. Provides excellent compatibility and respectable performance.

## Features

- **Unified API**: Write code once, run anywhere (Browser or Node.js).
- **Type Safe**: Full TypeScript definitions included.
- **Zero-Copy**: Optimized buffer handling for large files.
- **Native Performance**: Automatically upgrades to native C++/Rust speed in Node.js.
- **Helper Functions**: Utilities for File, Blob, URL, and Buffer inputs.

## Installation

```bash
npm install ply2splat
```

*Note: The native bindings (`ply2splat-native`) are an optional dependency. npm will try to install them automatically. If installation fails (e.g., unsupported OS), `ply2splat` will transparently fall back to the WASM implementation.*

## Usage

### Browser

```typescript
import { init, convertFromFile, convertFromUrl, downloadSplat } from 'ply2splat';

// Initialize (loads WASM)
await init();

// Convert from a File input (e.g., drag-and-drop)
async function handleFile(file: File) {
  const result = await convertFromFile(file);
  console.log(`Converted ${result.count} splats`);
  
  // Download the .splat file
  downloadSplat(result.data, 'output.splat');
  
  // Free memory when done
  result.free();
}
```

### Node.js

```typescript
import { init, convertFromBuffer, getBackend } from 'ply2splat';
import { readFileSync, writeFileSync } from 'fs';

// Initialize (loads Native if available, else WASM)
await init();

console.log(`Active Backend: ${getBackend()}`); // 'native' or 'wasm'

// Read PLY file
const plyBuffer = readFileSync('input.ply');

// Convert
const result = convertFromBuffer(plyBuffer);
console.log(`Converted ${result.count} splats`);

// Write SPLAT file
writeFileSync('output.splat', result.data);

// Free memory (important for WASM backend)
result.free();
```

## Native CLI

This package also enables running the high-performance Rust CLI directly via `npx` without compiling from source:

```bash
npx ply2splat-native --input model.ply --output model.splat
```

## API Reference

### `init(): Promise<void>`
Initializes the library. Must be called before any conversion functions.

### `convert(data: Uint8Array, options?: ConvertOptions): ConversionResult`
Core conversion function.
- `data`: Raw PLY file bytes.
- `options`: `{ sort?: boolean }` (default: `true`). Sorting improves rendering quality but takes longer.

### `getBackend(): 'native' | 'wasm' | null`
Returns the currently active backend.

### Helper Functions
- `convertFromFile(file: File, options?)`
- `convertFromUrl(url: string | URL, options?)`
- `convertFromBuffer(buffer: Buffer, options?)`
- `convertFromBlob(blob: Blob, options?)`
