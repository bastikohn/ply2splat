/**
 * @ply2splat/browser - Browser bindings for ply2splat
 *
 * High-level, asynchronous API for converting PLY files to SPLAT format in the browser.
 * Handles Web Workers and SharedArrayBuffer automatically.
 *
 * @example
 * ```typescript
 * import { initWasm, convert } from "@ply2splat/browser";
 *
 * // Initialize once (optional - convert() will auto-initialize)
 * await initWasm();
 *
 * // Convert PLY to SPLAT
 * const plyData = new Uint8Array(await file.arrayBuffer());
 * const result = await convert(plyData, { sort: true });
 *
 * // Download the result
 * const blob = new Blob([result.data], { type: "application/octet-stream" });
 * ```
 *
 * @module
 */

export type { ConversionResult, InitOptions, ConvertOptions } from "./types";

import type {
  ConversionResult,
  InitOptions,
  ConvertOptions,
  WorkerRequest,
  WorkerResponse,
} from "./types";
import wasmUrl from "@ply2splat/native-wasm32-wasi/ply2splat-native.wasm32-wasi.wasm?url";

const defaultMainWorkerUrl = new URL("./worker.js", import.meta.url).toString();
const defaultWasiWorkerUrl = new URL(
  "./wasi-worker.js",
  import.meta.url
).toString();

// Asset URLs - these need to be set before initialization
let configuredWasmUrl: string = wasmUrl;
let configuredMainWorkerUrl: string = defaultMainWorkerUrl;
let configuredWasiWorkerUrl: string = defaultWasiWorkerUrl;
let configuredAsyncWorkPoolSize = 4;

/**
 * Configure asset URLs for the WASM module.
 * Call this before initWasm() or convert() to set custom URLs.
 *
 * This is useful when your bundler needs to handle the asset URLs differently,
 * or when you're serving assets from a CDN.
 *
 * @example
 * ```typescript
 * import { configure, initWasm } from "@ply2splat/browser";
 *
 * // For Vite, you might use:
 * import wasmUrl from "@ply2splat/native-wasm32-wasi/ply2splat-native.wasm32-wasi.wasm?url";
 * import workerUrl from "@ply2splat/browser/worker?worker&url";
 * import wasiWorkerUrl from "@ply2splat/browser/wasi-worker?worker&url";
 *
 * configure({
 *   wasmUrl,
 *   mainWorkerUrl: workerUrl,
 *   wasiWorkerUrl,
 * });
 *
 * await initWasm();
 * ```
 */
export function configure(options: {
  wasmUrl?: string;
  mainWorkerUrl?: string;
  wasiWorkerUrl?: string;
  asyncWorkPoolSize?: number;
}): void {
  if (options.wasmUrl) configuredWasmUrl = options.wasmUrl;
  if (options.mainWorkerUrl) configuredMainWorkerUrl = options.mainWorkerUrl;
  if (options.wasiWorkerUrl) configuredWasiWorkerUrl = options.wasiWorkerUrl;
  if (options.asyncWorkPoolSize !== undefined) {
    configuredAsyncWorkPoolSize = options.asyncWorkPoolSize;
  }
}

/**
 * Ply2Splat client instance created by createClient().
 */
export interface Ply2SplatClient {
  /** Initialize the WASM module */
  initWasm(options?: InitOptions): Promise<void>;
  /** Convert PLY data to SPLAT format */
  convert(
    plyData: Uint8Array,
    options?: ConvertOptions
  ): Promise<ConversionResult>;
  /** Terminate the worker and clean up resources */
  terminate(): void;
  /** Check if the WASM module is initialized */
  isInitialized(): boolean;
}

/**
 * Create a new Ply2Splat client instance.
 *
 * This is an alternative to the module-level functions that allows
 * you to create multiple independent instances or configure URLs
 * at creation time.
 *
 * @example
 * ```typescript
 * import { createClient } from "@ply2splat/browser";
 * import wasmUrl from "@ply2splat/native-wasm32-wasi/ply2splat-native.wasm32-wasi.wasm?url";
 * import workerUrl from "@ply2splat/browser/worker?worker&url";
 * import wasiWorkerUrl from "@ply2splat/browser/wasi-worker?worker&url";
 *
 * const client = createClient({
 *   wasmUrl,
 *   mainWorkerUrl: workerUrl,
 *   wasiWorkerUrl,
 * });
 *
 * const result = await client.convert(plyData);
 * client.terminate();
 * ```
 */
export function createClient(
  config: {
    wasmUrl?: string;
    mainWorkerUrl?: string;
    wasiWorkerUrl?: string;
    asyncWorkPoolSize?: number;
  } = {}
): Ply2SplatClient {
  const wasmUrl = config.wasmUrl ?? configuredWasmUrl;
  const mainWorkerUrl = config.mainWorkerUrl ?? configuredMainWorkerUrl;
  const wasiWorkerUrl = config.wasiWorkerUrl ?? configuredWasiWorkerUrl;
  const asyncWorkPoolSize =
    config.asyncWorkPoolSize ?? configuredAsyncWorkPoolSize;
  let clientWorker: Worker | null = null;
  let clientMessageId = 0;
  let clientInitialized = false;
  const clientPendingMessages = new Map<
    number,
    { resolve: (value: unknown) => void; reject: (error: Error) => void }
  >();

  function getClientWorker(): Worker {
    if (!clientWorker) {
      console.log("[ply2splat client] Creating worker...");
      clientWorker = new Worker(mainWorkerUrl, { type: "module" });

      clientWorker.onmessage = (e: MessageEvent<WorkerResponse>) => {
        const { type, id, result, error } = e.data;
        console.log("[ply2splat client] Worker message received:", type, id);

        const pending = clientPendingMessages.get(id);
        if (pending) {
          clientPendingMessages.delete(id);
          if (type === "error") {
            pending.reject(new Error(error ?? "Unknown worker error"));
          } else {
            pending.resolve(result);
          }
        }
      };

      clientWorker.onerror = (e) => {
        console.error("[ply2splat client] Worker error:", e);
      };
    }
    return clientWorker;
  }

  function clientPostMessage<T>(
    type: WorkerRequest["type"],
    payload?: WorkerRequest["payload"]
  ): Promise<T> {
    return new Promise((resolve, reject) => {
      const id = clientMessageId++;
      console.log("[ply2splat client] Posting message:", type, id);
      clientPendingMessages.set(id, {
        resolve: resolve as (value: unknown) => void,
        reject,
      });
      getClientWorker().postMessage({ type, id, payload } as WorkerRequest);
    });
  }

  return {
    async initWasm(options?: InitOptions): Promise<void> {
      if (clientInitialized) return;

      console.log("[ply2splat client] Initializing WASM...");
      await clientPostMessage("init", {
        wasmUrl: options?.wasmUrl ?? wasmUrl,
        wasiWorkerUrl: options?.wasiWorkerUrl ?? wasiWorkerUrl,
        asyncWorkPoolSize: options?.asyncWorkPoolSize ?? asyncWorkPoolSize,
      });
      clientInitialized = true;
      console.log("[ply2splat client] WASM initialized");
    },

    async convert(
      plyData: Uint8Array,
      options?: ConvertOptions
    ): Promise<ConversionResult> {
      if (!clientInitialized) {
        await this.initWasm();
      }
      console.log("[ply2splat client] Converting PLY data...");
      return clientPostMessage<ConversionResult>("convert", {
        plyData,
        sort: options?.sort ?? true,
      });
    },

    terminate(): void {
      if (clientWorker) {
        clientWorker.terminate();
        clientWorker = null;
        clientInitialized = false;
        clientPendingMessages.clear();
        console.log("[ply2splat client] Worker terminated");
      }
    },

    isInitialized(): boolean {
      return clientInitialized;
    },
  };
}
