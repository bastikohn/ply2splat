import { Buffer } from "buffer";

// Polyfill Buffer for browser environment (required by emnapi)
if (typeof globalThis.Buffer === "undefined") {
  globalThis.Buffer = Buffer;
}
