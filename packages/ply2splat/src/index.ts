/**
 * ply2splat - Convert Gaussian Splatting PLY files to SPLAT format
 * 
 * This is the unified npm package that provides both native and WASM bindings.
 * Native bindings are preferred when available (better performance), falling back to WASM.
 * 
 * @packageDocumentation
 */

// Types
export interface Splat {
  /** Position [x, y, z] in world coordinates */
  position: [number, number, number];
  /** Scale [x, y, z] - exponential scale values */
  scale: [number, number, number];
  /** Color [R, G, B, A] - values 0-255 */
  color: [number, number, number, number];
  /** Rotation quaternion [r0, r1, r2, r3] - encoded as 0-255 */
  rotation: [number, number, number, number];
}

export interface ConvertOptions {
  /** 
   * Whether to sort splats by importance (volume * opacity).
   * Sorting improves rendering quality but takes longer.
   * @default true
   */
  sort?: boolean;
}

export interface ConversionResult {
  /** Raw SPLAT binary data (32 bytes per splat) */
  readonly data: Uint8Array;
  /** Number of splats in the result */
  readonly count: number;
  /** Parse the data into typed Splat objects */
  toSplats(): Splat[];
  /** Free the underlying memory (if applicable) */
  free(): void;
}

export type PlyInput = 
  | Uint8Array 
  | ArrayBuffer 
  | Blob 
  | File 
  | Buffer
  | Response;

// Internal binding interface
interface NativeBinding {
  convert(data: Buffer, sort?: boolean): { data: Buffer; count: number };
  getSplatCount(data: Buffer): number;
  isNative: true;
}

interface WasmBinding {
  convert(data: Uint8Array, sort?: boolean): { data: Uint8Array; count: number; free(): void };
  parseSplatData(data: Uint8Array): Array<{ position: number[]; scale: number[]; color: number[]; rotation: number[] }>;
  getSplatCount(data: Uint8Array): number;
  isNative: false;
  init(): Promise<void>;
  initSync?(wasmModule: WebAssembly.Module): void;
}

type Binding = NativeBinding | WasmBinding;

let binding: Binding | null = null;
let initPromise: Promise<void> | null = null;

/**
 * Try to load native bindings
 */
function loadNativeBinding(): NativeBinding | null {
  try {
    // Try to require the native binding
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const native = require('@ply2splat/native') as NativeBinding;
    native.isNative = true;
    return native;
  } catch {
    return null;
  }
}

/**
 * Load WASM bindings
 */
async function loadWasmBinding(): Promise<WasmBinding> {
  const wasm = await import('../wasm/ply2splat_wasm.js');
  await wasm.default();
  return {
    convert: wasm.convert,
    parseSplatData: wasm.parseSplatData,
    getSplatCount: wasm.getSplatCount,
    isNative: false,
    init: async () => { await wasm.default(); },
    initSync: wasm.initSync,
  };
}

/**
 * Initialize the ply2splat module.
 * Attempts to load native bindings first, falling back to WASM if unavailable.
 * 
 * @returns Promise that resolves when initialization is complete
 */
export async function init(): Promise<void> {
  if (binding) return;
  if (initPromise) return initPromise;
  
  initPromise = (async () => {
    // Try native first
    const native = loadNativeBinding();
    if (native) {
      binding = native;
      return;
    }
    
    // Fall back to WASM
    binding = await loadWasmBinding();
  })();
  
  return initPromise;
}

/**
 * Check if native bindings are being used.
 * 
 * @returns 'native' if using native bindings, 'wasm' if using WASM
 */
export function getBackend(): 'native' | 'wasm' | null {
  if (!binding) return null;
  return binding.isNative ? 'native' : 'wasm';
}

/**
 * Parse SPLAT binary data into typed Splat objects.
 */
function parseSplatDataFromBytes(data: Uint8Array): Splat[] {
  if (data.length % 32 !== 0) {
    throw new Error(`Invalid SPLAT data: size ${data.length} is not a multiple of 32 bytes`);
  }
  
  const splats: Splat[] = [];
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  
  for (let i = 0; i < data.length; i += 32) {
    splats.push({
      position: [
        view.getFloat32(i, true),
        view.getFloat32(i + 4, true),
        view.getFloat32(i + 8, true),
      ],
      scale: [
        view.getFloat32(i + 12, true),
        view.getFloat32(i + 16, true),
        view.getFloat32(i + 20, true),
      ],
      color: [
        data[i + 24],
        data[i + 25],
        data[i + 26],
        data[i + 27],
      ],
      rotation: [
        data[i + 28],
        data[i + 29],
        data[i + 30],
        data[i + 31],
      ],
    });
  }
  
  return splats;
}

/**
 * Convert various input types to Uint8Array.
 */
export async function toUint8Array(input: PlyInput): Promise<Uint8Array> {
  if (input instanceof Uint8Array) {
    return input;
  }

  if (input instanceof ArrayBuffer) {
    return new Uint8Array(input);
  }

  if (typeof Response !== 'undefined' && input instanceof Response) {
    const buffer = await input.arrayBuffer();
    return new Uint8Array(buffer);
  }

  if (typeof Blob !== 'undefined' && input instanceof Blob) {
    const buffer = await input.arrayBuffer();
    return new Uint8Array(buffer);
  }

  if (typeof Buffer !== 'undefined' && Buffer.isBuffer(input)) {
    return new Uint8Array(input.buffer, input.byteOffset, input.byteLength);
  }

  throw new Error(`Unsupported input type: ${typeof input}`);
}

/**
 * Convert PLY data to SPLAT format.
 * 
 * @param plyData - PLY file data as Uint8Array
 * @param options - Conversion options
 * @returns ConversionResult with data and helper methods
 */
export function convert(plyData: Uint8Array, options?: ConvertOptions): ConversionResult {
  if (!binding) {
    throw new Error('ply2splat not initialized. Call init() first.');
  }
  
  const sort = options?.sort ?? true;
  
  if (binding.isNative) {
    const result = binding.convert(Buffer.from(plyData), sort);
    const data = new Uint8Array(result.data);
    return {
      data,
      count: result.count,
      toSplats: () => parseSplatDataFromBytes(data),
      free: () => {},
    };
  } else {
    const result = binding.convert(plyData, sort);
    return {
      get data() { return result.data; },
      get count() { return result.count; },
      toSplats: () => parseSplatDataFromBytes(result.data),
      free: () => result.free?.(),
    };
  }
}

/**
 * Get the number of splats in SPLAT binary data.
 */
export function getSplatCount(splatData: Uint8Array): number {
  if (splatData.length % 32 !== 0) {
    throw new Error(`Invalid SPLAT data: size ${splatData.length} is not a multiple of 32 bytes`);
  }
  return splatData.length / 32;
}

/**
 * Parse SPLAT binary data into typed Splat objects.
 */
export function parseSplatData(splatData: Uint8Array): Splat[] {
  return parseSplatDataFromBytes(splatData);
}

// Helper functions for various input types

export async function convertFromBlob(blob: Blob, options?: ConvertOptions): Promise<ConversionResult> {
  const data = await toUint8Array(blob);
  return convert(data, options);
}

export async function convertFromFile(file: File, options?: ConvertOptions): Promise<ConversionResult> {
  return convertFromBlob(file, options);
}

export function convertFromArrayBuffer(buffer: ArrayBuffer, options?: ConvertOptions): ConversionResult {
  return convert(new Uint8Array(buffer), options);
}

export async function convertFromResponse(response: Response, options?: ConvertOptions): Promise<ConversionResult> {
  const data = await toUint8Array(response);
  return convert(data, options);
}

export async function convertFromUrl(url: string | URL, options?: ConvertOptions): Promise<ConversionResult> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to fetch PLY file: ${response.status} ${response.statusText}`);
  }
  return convertFromResponse(response, options);
}

export function convertFromBuffer(buffer: Buffer, options?: ConvertOptions): ConversionResult {
  const data = new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  return convert(data, options);
}

export async function convertFromFormData(
  formData: FormData,
  fieldName: string = 'file',
  options?: ConvertOptions
): Promise<ConversionResult> {
  const file = formData.get(fieldName);
  if (!file) {
    throw new Error(`No file found in FormData field '${fieldName}'`);
  }
  if (!(file instanceof Blob)) {
    throw new Error(`FormData field '${fieldName}' is not a Blob or File`);
  }
  return convertFromBlob(file, options);
}

export function createSplatBlob(splatData: Uint8Array): Blob {
  return new Blob([splatData.slice()], { type: 'application/octet-stream' });
}

export function downloadSplat(splatData: Uint8Array, filename: string = 'output.splat'): void {
  if (typeof document === 'undefined') {
    throw new Error('downloadSplat is only available in browser environments');
  }
  
  const blob = createSplatBlob(splatData);
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}
