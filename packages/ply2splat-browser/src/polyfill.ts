import { Buffer } from "buffer";

// Extend globalThis to include Buffer
declare global {
  // eslint-disable-next-line no-var
  var Buffer: typeof import("buffer").Buffer | undefined;
}

/**
 * Polyfill Buffer for browser environment.
 * Required by emnapi (the N-API implementation for WebAssembly).
 * @internal
 */
export function installBufferPolyfill(): void {
  if (typeof globalThis.Buffer === "undefined") {
    globalThis.Buffer = Buffer;
  }
}

// Auto-install on import
installBufferPolyfill();

