// Worker for running ply2splat WASM conversion
import {
  getDefaultContext as __emnapiGetDefaultContext,
  instantiateNapiModule as __emnapiInstantiateNapiModule,
  WASI as __WASI,
} from "@napi-rs/wasm-runtime";

let wasmModule = null;

async function initWasm(wasmUrl, wasiWorkerUrl) {
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
    __wasmFile.byteLength,
  );

  // Convert relative URL to absolute URL for the child worker
  const absoluteWorkerUrl = new URL(wasiWorkerUrl, self.location.href).href;
  console.log("[ply2splat worker] Child worker URL:", absoluteWorkerUrl);

  console.log("[ply2splat worker] Instantiating NAPI module (async)...");
  const { napiModule: __napiModule } = await __emnapiInstantiateNapiModule(
    __wasmFile,
    {
      context: __emnapiContext,
      // asyncWorkPoolSize: 0, // Disable thread pool - not needed for conversion
      wasi: __wasi,
      onCreateWorker() {
        // This shouldn't be called with asyncWorkPoolSize: 0
        console.log(
          "[ply2splat worker] Creating child worker at:",
          absoluteWorkerUrl,
        );
      },
      overwriteImports(importObject) {
        importObject.env = {
          ...importObject.env,
          ...importObject.napi,
          ...importObject.emnapi,
          memory: __sharedMemory,
        };
        return importObject;
      },
      beforeInit({ instance }) {
        for (const name of Object.keys(instance.exports)) {
          if (name.startsWith("__napi_register__")) {
            instance.exports[name]();
          }
        }
      },
    },
  );

  wasmModule = __napiModule.exports;
  console.log("[ply2splat worker] WASM initialized successfully");
}

self.onmessage = async (e) => {
  const { type, id, payload } = e.data;
  console.log("[ply2splat worker] Received message:", type, id);

  try {
    if (type === "init") {
      await initWasm(payload.wasmUrl, payload.wasiWorkerUrl);
      console.log("[ply2splat worker] Sending init-complete");
      self.postMessage({ type: "init-complete", id });
    } else if (type === "convert") {
      if (!wasmModule) {
        throw new Error("WASM module not initialized");
      }
      const resultSimpleFn = wasmModule.simpleFn();
      console.log("[ply2splat worker] simpleFn result:", resultSimpleFn);
      console.log(
        "[ply2splat worker] Converting PLY data, size:",
        payload.plyData.byteLength,
      );
      const result = wasmModule.convert(payload.plyData, payload.sort ?? true);
      console.log(
        "[ply2splat worker] Conversion complete, splats:",
        result.count,
      );
      // Transfer the buffer back to main thread
      self.postMessage(
        {
          type: "convert-complete",
          id,
          result: { data: result.data, count: result.count },
        },
        [result.data.buffer],
      );
    }
  } catch (error) {
    console.error("[ply2splat worker] Error:", error);
    self.postMessage({ type: "error", id, error: error.message });
  }
};
