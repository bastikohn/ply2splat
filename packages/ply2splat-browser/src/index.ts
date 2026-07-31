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

interface ClientConfig {
  wasmUrl?: string;
  mainWorkerUrl?: string;
  wasiWorkerUrl?: string;
  asyncWorkPoolSize?: number;
}

const defaultMainWorkerUrl = new URL("./worker.js", import.meta.url).toString();
const defaultWasiWorkerUrl = new URL(
  "./wasi-worker.js",
  import.meta.url
).toString();
const defaultAsyncWorkPoolSize = 4;

// Asset URLs - these need to be set before initialization
let configuredWasmUrl: string = wasmUrl;
let configuredMainWorkerUrl: string = defaultMainWorkerUrl;
let configuredWasiWorkerUrl: string = defaultWasiWorkerUrl;
let configuredAsyncWorkPoolSize = defaultAsyncWorkPoolSize;
let defaultClient: Ply2SplatClient | null = null;

function normalizeAsyncWorkPoolSize(size: number): number {
  if (!Number.isInteger(size) || size < 1) {
    throw new RangeError("asyncWorkPoolSize must be a positive integer");
  }
  return size;
}

function resolveClientConfig(config: ClientConfig = {}): Required<ClientConfig> {
  return {
    wasmUrl: config.wasmUrl ?? configuredWasmUrl,
    mainWorkerUrl: config.mainWorkerUrl ?? configuredMainWorkerUrl,
    wasiWorkerUrl: config.wasiWorkerUrl ?? configuredWasiWorkerUrl,
    asyncWorkPoolSize:
      config.asyncWorkPoolSize !== undefined
        ? normalizeAsyncWorkPoolSize(config.asyncWorkPoolSize)
        : configuredAsyncWorkPoolSize,
  };
}

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
export function configure(options: ClientConfig): void {
  if (options.wasmUrl) configuredWasmUrl = options.wasmUrl;
  if (options.mainWorkerUrl) configuredMainWorkerUrl = options.mainWorkerUrl;
  if (options.wasiWorkerUrl) configuredWasiWorkerUrl = options.wasiWorkerUrl;
  if (options.asyncWorkPoolSize !== undefined) {
    configuredAsyncWorkPoolSize = normalizeAsyncWorkPoolSize(
      options.asyncWorkPoolSize
    );
  }

  if (defaultClient) {
    defaultClient.terminate();
    defaultClient = null;
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
export function createClient(config: ClientConfig = {}): Ply2SplatClient {
  const {
    wasmUrl,
    mainWorkerUrl,
    wasiWorkerUrl,
    asyncWorkPoolSize,
  } = resolveClientConfig(config);
  let clientWorker: Worker | null = null;
  let clientMessageId = 0;
  let clientInitialized = false;
  let clientInitPromise: Promise<void> | null = null;
  const clientPendingMessages = new Map<
    number,
    { resolve: (value: unknown) => void; reject: (error: Error) => void }
  >();

  function rejectPendingMessages(error: Error): void {
    for (const pending of clientPendingMessages.values()) {
      pending.reject(error);
    }
    clientPendingMessages.clear();
  }

  function getClientWorker(): Worker {
    if (!clientWorker) {
      clientWorker = new Worker(mainWorkerUrl, { type: "module" });

      clientWorker.onmessage = (e: MessageEvent<WorkerResponse>) => {
        const response = e.data;
        const pending = clientPendingMessages.get(response.id);
        if (!pending) return;

        clientPendingMessages.delete(response.id);
        if (response.type === "error") {
          pending.reject(new Error(response.error));
        } else if (response.type === "convert-complete") {
          pending.resolve(response.result);
        } else {
          pending.resolve(undefined);
        }
      };

      clientWorker.onerror = (e) => {
        const error = new Error(e.message || "Worker error");
        rejectPendingMessages(error);
        clientWorker?.terminate();
        clientWorker = null;
        clientInitialized = false;
        clientInitPromise = null;
      };

      clientWorker.onmessageerror = () => {
        const error = new Error("Worker message could not be deserialized");
        rejectPendingMessages(error);
        clientWorker?.terminate();
        clientWorker = null;
        clientInitialized = false;
        clientInitPromise = null;
      };
    }
    return clientWorker;
  }

  function clientPostMessage<T>(
    request: Omit<WorkerRequest, "id">
  ): Promise<T> {
    return new Promise((resolve, reject) => {
      const id = clientMessageId++;
      clientPendingMessages.set(id, {
        resolve: resolve as (value: unknown) => void,
        reject,
      });
      try {
        getClientWorker().postMessage({ ...request, id } as WorkerRequest);
      } catch (error) {
        clientPendingMessages.delete(id);
        reject(error instanceof Error ? error : new Error(String(error)));
      }
    });
  }

  async function initClientWasm(options?: InitOptions): Promise<void> {
    if (clientInitialized) return;
    if (clientInitPromise) return clientInitPromise;

    clientInitPromise = clientPostMessage<void>({
      type: "init",
      payload: {
        wasmUrl: options?.wasmUrl ?? wasmUrl,
        wasiWorkerUrl: options?.wasiWorkerUrl ?? wasiWorkerUrl,
        asyncWorkPoolSize:
          options?.asyncWorkPoolSize !== undefined
            ? normalizeAsyncWorkPoolSize(options.asyncWorkPoolSize)
            : asyncWorkPoolSize,
      },
    })
      .then(() => {
        clientInitialized = true;
      })
      .finally(() => {
        clientInitPromise = null;
      });

    return clientInitPromise;
  }

  async function convertClient(
    plyData: Uint8Array,
    options?: ConvertOptions
  ): Promise<ConversionResult> {
    if (!clientInitialized) {
      await initClientWasm();
    }

    return clientPostMessage<ConversionResult>({
      type: "convert",
      payload: {
        plyData,
        sort: options?.sort ?? true,
      },
    });
  }

  return {
    initWasm: initClientWasm,
    convert: convertClient,

    terminate(): void {
      rejectPendingMessages(new Error("Worker terminated"));
      if (clientWorker) {
        clientWorker.terminate();
        clientWorker = null;
      }
      clientInitialized = false;
      clientInitPromise = null;
    },

    isInitialized(): boolean {
      return clientInitialized;
    },
  };
}

function getDefaultClient(): Ply2SplatClient {
  if (!defaultClient) {
    defaultClient = createClient();
  }
  return defaultClient;
}

export async function initWasm(options?: InitOptions): Promise<void> {
  return getDefaultClient().initWasm(options);
}

export async function convert(
  plyData: Uint8Array,
  options?: ConvertOptions
): Promise<ConversionResult> {
  return getDefaultClient().convert(plyData, options);
}

export function terminate(): void {
  if (defaultClient) {
    defaultClient.terminate();
    defaultClient = null;
  }
}

export function isInitialized(): boolean {
  return defaultClient?.isInitialized() ?? false;
}
