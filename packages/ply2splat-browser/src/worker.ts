/**
 * Main conversion worker for ply2splat.
 * This worker handles WASM initialization and PLY to SPLAT conversion.
 * @module
 */

// IMPORTANT: Import polyfill BEFORE @napi-rs/wasm-runtime
import "./polyfill";

import {
  getDefaultContext as __emnapiGetDefaultContext,
  instantiateNapiModule as __emnapiInstantiateNapiModule,
  WASI as __WASI,
  type ImportObject,
} from "@napi-rs/wasm-runtime";

import type { WorkerRequest, WorkerResponse, ConversionResult } from "./types";

interface WasmModule {
  convert(data: Uint8Array, sort: boolean): { data: Uint8Array; count: number };
  getSplatCount(data: Uint8Array): number;
  simpleFn(): number;
}

let wasmModule: WasmModule | null = null;

async function initWasm(
  wasmUrl: string,
  wasiWorkerUrl: string,
  asyncWorkPoolSize: number = 4
): Promise<void> {
  if (wasmModule) return;

  console.log("[ply2splat worker] Initializing WASM...", {
    wasmUrl,
    wasiWorkerUrl,
  });

  const __wasi = new __WASI({
    version: "preview1",
  });

  const __emnapiContext = __emnapiGetDefaultContext();

  const __sharedMemory = new WebAssembly.Memory({
    initial: 4000,
    maximum: 65536,
    shared: true,
  });

  console.log("[ply2splat worker] Fetching WASM file...");
  const __wasmFile = await fetch(wasmUrl).then((res) => res.arrayBuffer());
  console.log(
    "[ply2splat worker] WASM file fetched, size:",
    __wasmFile.byteLength
  );

  // Convert relative URL to absolute URL for the child worker
  const absoluteWorkerUrl = new URL(wasiWorkerUrl, self.location.href).href;
  console.log("[ply2splat worker] Child worker URL:", absoluteWorkerUrl);

  console.log("[ply2splat worker] Instantiating NAPI module (async)...");
  const { napiModule: __napiModule } = await __emnapiInstantiateNapiModule(
    __wasmFile,
    {
      context: __emnapiContext,
      asyncWorkPoolSize,
      wasi: __wasi,
      onCreateWorker() {
        console.log(
          "[ply2splat worker] Creating child worker at:",
          absoluteWorkerUrl
        );
        const worker = new Worker(absoluteWorkerUrl, {
          type: "module",
        });
        return worker;
      },
      overwriteImports(importObject: ImportObject) {
        importObject.env = {
          ...importObject.env,
          ...importObject.napi,
          ...importObject.emnapi,
          memory: __sharedMemory,
        };
        return importObject;
      },
      beforeInit({ instance }: { instance: WebAssembly.Instance }) {
        for (const name of Object.keys(instance.exports)) {
          if (name.startsWith("__napi_register__")) {
            (instance.exports[name] as () => void)();
          }
        }
      },
    }
  );

  wasmModule = __napiModule.exports as unknown as WasmModule;
  console.log("[ply2splat worker] WASM initialized successfully");
}

function sendResponse(response: WorkerResponse): void {
  if (response.type === "convert-complete" && response.result) {
    // Transfer the buffer back to main thread for better performance
    self.postMessage(response, [response.result.data.buffer]);
  } else {
    self.postMessage(response);
  }
}

self.onmessage = async (e: MessageEvent<WorkerRequest>) => {
  const { type, id, payload } = e.data;
  console.log("[ply2splat worker] Received message:", type, id);

  try {
    if (type === "init") {
      if (!payload?.wasmUrl || !payload?.wasiWorkerUrl) {
        throw new Error("wasmUrl and wasiWorkerUrl are required for init");
      }
      await initWasm(
        payload.wasmUrl,
        payload.wasiWorkerUrl,
        payload.asyncWorkPoolSize ?? 4
      );
      console.log("[ply2splat worker] Sending init-complete");
      sendResponse({ type: "init-complete", id });
    } else if (type === "convert") {
      if (!wasmModule) {
        throw new Error("WASM module not initialized");
      }
      if (!payload?.plyData) {
        throw new Error("plyData is required for convert");
      }

      const simpleResult = wasmModule.simpleFn();
      console.log(`Simple result: `, simpleResult);
      console.log(
        "[ply2splat worker] Converting PLY data, size:",
        payload.plyData.byteLength
      );

      const result = wasmModule.convert(payload.plyData, payload.sort ?? true);

      console.log(
        "[ply2splat worker] Conversion complete, splats:",
        result.count
      );

      // Create a copy of the data because the original is in SharedArrayBuffer
      // which cannot be transferred.
      const dataCopy = new Uint8Array(result.data);

      const response: WorkerResponse = {
        type: "convert-complete",
        id,
        result: { data: dataCopy, count: result.count },
      };
      sendResponse(response);
    }
  } catch (error) {
    console.error("[ply2splat worker] Error:", error);
    sendResponse({
      type: "error",
      id,
      error: error instanceof Error ? error.message : String(error),
    });
  }
};
