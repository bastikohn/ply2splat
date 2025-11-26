/**
 * TypeScript helpers for ply2splat WASM module.
 * 
 * This file provides type-safe wrappers and helper functions for working with
 * various input types (Blob, File, Node Buffer, etc.) when using ply2splat.
 * 
 * @packageDocumentation
 */

// Re-export everything from the generated WASM bindings
export {
  ConversionResult,
  InitInput,
  InitOutput,
  SyncInitInput,
  initSync,
} from './ply2splat.js';

export { default as init } from './ply2splat.js';

// Import WASM functions for internal use
import {
  convert as wasmConvert,
  parseSplatData as wasmParseSplatData,
  getSplatCount as wasmGetSplatCount,
} from './ply2splat.js';

/**
 * A single Gaussian Splat with position, scale, color, and rotation.
 */
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

/**
 * Options for PLY to SPLAT conversion.
 */
export interface ConvertOptions {
  /** 
   * Whether to sort splats by importance (volume * opacity).
   * Sorting improves rendering quality but takes longer.
   * @default true
   */
  sort?: boolean;
}

/**
 * Result of a PLY to SPLAT conversion with typed accessors.
 */
export interface TypedConversionResult {
  /** Raw SPLAT binary data (32 bytes per splat) */
  readonly data: Uint8Array;
  /** Number of splats in the result */
  readonly count: number;
  /** Parse the data into typed Splat objects */
  toSplats(): Splat[];
  /** Free the underlying WASM memory (optional, will be GC'd if not called) */
  free(): void;
}

/**
 * Input types that can be converted to Uint8Array for PLY data.
 */
export type PlyInput = 
  | Uint8Array 
  | ArrayBuffer 
  | Blob 
  | File 
  | Buffer
  | Response;

/**
 * Convert various input types to Uint8Array.
 * Supports: Uint8Array, ArrayBuffer, Blob, File, Node Buffer, and Response.
 * 
 * @param input - The PLY data in various formats
 * @returns Promise resolving to Uint8Array
 */
export async function toUint8Array(input: PlyInput): Promise<Uint8Array> {
  // Already Uint8Array
  if (input instanceof Uint8Array) {
    return input;
  }

  // ArrayBuffer
  if (input instanceof ArrayBuffer) {
    return new Uint8Array(input);
  }

  // Response (from fetch)
  if (typeof Response !== 'undefined' && input instanceof Response) {
    const buffer = await input.arrayBuffer();
    return new Uint8Array(buffer);
  }

  // Blob or File (browser)
  if (typeof Blob !== 'undefined' && input instanceof Blob) {
    const buffer = await input.arrayBuffer();
    return new Uint8Array(buffer);
  }

  // Node.js Buffer
  if (typeof Buffer !== 'undefined' && Buffer.isBuffer(input)) {
    return new Uint8Array(input.buffer, input.byteOffset, input.byteLength);
  }

  throw new Error(`Unsupported input type: ${typeof input}`);
}

/**
 * Parse SPLAT binary data into typed Splat objects.
 * 
 * @param splatData - Raw SPLAT data as Uint8Array (must be multiple of 32 bytes)
 * @returns Array of Splat objects with proper TypeScript types
 * @throws Error if data length is not a multiple of 32 bytes
 */
export function parseSplatData(splatData: Uint8Array): Splat[] {
  const rawSplats = wasmParseSplatData(splatData) as Array<{
    position: number[];
    scale: number[];
    color: number[];
    rotation: number[];
  }>;

  return rawSplats.map(raw => ({
    position: [raw.position[0], raw.position[1], raw.position[2]] as [number, number, number],
    scale: [raw.scale[0], raw.scale[1], raw.scale[2]] as [number, number, number],
    color: [raw.color[0], raw.color[1], raw.color[2], raw.color[3]] as [number, number, number, number],
    rotation: [raw.rotation[0], raw.rotation[1], raw.rotation[2], raw.rotation[3]] as [number, number, number, number],
  }));
}

/**
 * Get the number of splats in SPLAT binary data.
 * 
 * @param splatData - Raw SPLAT data as Uint8Array
 * @returns Number of splats in the data
 * @throws Error if data length is not a multiple of 32 bytes
 */
export function getSplatCount(splatData: Uint8Array): number {
  return wasmGetSplatCount(splatData);
}

/**
 * Convert PLY data to SPLAT format from Uint8Array.
 * 
 * @param plyData - PLY file data as Uint8Array
 * @param options - Conversion options
 * @returns TypedConversionResult with data and helper methods
 */
export function convert(plyData: Uint8Array, options?: ConvertOptions): TypedConversionResult {
  const sort = options?.sort ?? true;
  const result = wasmConvert(plyData, sort);
  
  return {
    get data() {
      return result.data;
    },
    get count() {
      return result.count;
    },
    toSplats() {
      return parseSplatData(result.data);
    },
    free() {
      result.free();
    },
  };
}

/**
 * Convert PLY data from a Blob to SPLAT format.
 * Useful for handling file inputs in the browser.
 * 
 * @param blob - Blob containing PLY data
 * @param options - Conversion options
 * @returns Promise resolving to TypedConversionResult
 * 
 * @example
 * ```typescript
 * const fileInput = document.querySelector('input[type="file"]');
 * fileInput.addEventListener('change', async (e) => {
 *   const file = e.target.files[0];
 *   const result = await convertFromBlob(file);
 *   console.log(`Converted ${result.count} splats`);
 * });
 * ```
 */
export async function convertFromBlob(blob: Blob, options?: ConvertOptions): Promise<TypedConversionResult> {
  const data = await toUint8Array(blob);
  return convert(data, options);
}

/**
 * Convert PLY data from a File to SPLAT format.
 * Alias for convertFromBlob since File extends Blob.
 * 
 * @param file - File containing PLY data
 * @param options - Conversion options
 * @returns Promise resolving to TypedConversionResult
 * 
 * @example
 * ```typescript
 * const input = document.getElementById('ply-file') as HTMLInputElement;
 * const file = input.files[0];
 * const result = await convertFromFile(file);
 * ```
 */
export async function convertFromFile(file: File, options?: ConvertOptions): Promise<TypedConversionResult> {
  return convertFromBlob(file, options);
}

/**
 * Convert PLY data from an ArrayBuffer to SPLAT format.
 * 
 * @param buffer - ArrayBuffer containing PLY data
 * @param options - Conversion options
 * @returns TypedConversionResult
 * 
 * @example
 * ```typescript
 * const response = await fetch('model.ply');
 * const buffer = await response.arrayBuffer();
 * const result = convertFromArrayBuffer(buffer);
 * ```
 */
export function convertFromArrayBuffer(buffer: ArrayBuffer, options?: ConvertOptions): TypedConversionResult {
  return convert(new Uint8Array(buffer), options);
}

/**
 * Convert PLY data from a fetch Response to SPLAT format.
 * 
 * @param response - Fetch Response containing PLY data
 * @param options - Conversion options
 * @returns Promise resolving to TypedConversionResult
 * 
 * @example
 * ```typescript
 * const response = await fetch('https://example.com/model.ply');
 * const result = await convertFromResponse(response);
 * ```
 */
export async function convertFromResponse(response: Response, options?: ConvertOptions): Promise<TypedConversionResult> {
  const data = await toUint8Array(response);
  return convert(data, options);
}

/**
 * Convert PLY data from a URL by fetching it first.
 * 
 * @param url - URL to fetch PLY data from
 * @param options - Conversion options
 * @returns Promise resolving to TypedConversionResult
 * 
 * @example
 * ```typescript
 * const result = await convertFromUrl('https://example.com/model.ply');
 * const splats = result.toSplats();
 * ```
 */
export async function convertFromUrl(url: string | URL, options?: ConvertOptions): Promise<TypedConversionResult> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to fetch PLY file: ${response.status} ${response.statusText}`);
  }
  return convertFromResponse(response, options);
}

/**
 * Convert PLY data from a Node.js Buffer to SPLAT format.
 * 
 * @param buffer - Node.js Buffer containing PLY data
 * @param options - Conversion options
 * @returns TypedConversionResult
 * 
 * @example
 * ```typescript
 * import { readFileSync } from 'fs';
 * const buffer = readFileSync('model.ply');
 * const result = convertFromBuffer(buffer);
 * ```
 */
export function convertFromBuffer(buffer: Buffer, options?: ConvertOptions): TypedConversionResult {
  const data = new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  return convert(data, options);
}

/**
 * Extract a file from FormData and convert it to SPLAT format.
 * Useful for server-side processing of uploaded files.
 * 
 * @param formData - FormData containing the PLY file
 * @param fieldName - Name of the form field containing the file (default: 'file')
 * @param options - Conversion options
 * @returns Promise resolving to TypedConversionResult
 * @throws Error if the field is not found or is not a File
 * 
 * @example
 * ```typescript
 * // Express.js example with multer
 * app.post('/convert', upload.single('ply'), async (req, res) => {
 *   const result = await convertFromFormData(req.body, 'ply');
 *   res.send(result.data);
 * });
 * ```
 */
export async function convertFromFormData(
  formData: FormData,
  fieldName: string = 'file',
  options?: ConvertOptions
): Promise<TypedConversionResult> {
  const file = formData.get(fieldName);
  if (!file) {
    throw new Error(`No file found in FormData field '${fieldName}'`);
  }
  if (!(file instanceof Blob)) {
    throw new Error(`FormData field '${fieldName}' is not a file`);
  }
  return convertFromBlob(file, options);
}

/**
 * Create a downloadable Blob from SPLAT data.
 * Useful for allowing users to download converted files.
 * 
 * @param splatData - SPLAT binary data
 * @returns Blob with application/octet-stream MIME type
 * 
 * @example
 * ```typescript
 * const result = convert(plyData);
 * const blob = createSplatBlob(result.data);
 * const url = URL.createObjectURL(blob);
 * downloadLink.href = url;
 * ```
 */
export function createSplatBlob(splatData: Uint8Array): Blob {
  return new Blob([splatData.slice().buffer], { type: 'application/octet-stream' });
}

/**
 * Trigger a download of SPLAT data in the browser.
 * 
 * @param splatData - SPLAT binary data
 * @param filename - Filename for the download (default: 'output.splat')
 * 
 * @example
 * ```typescript
 * const result = convert(plyData);
 * downloadSplat(result.data, 'my-model.splat');
 * ```
 */
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
