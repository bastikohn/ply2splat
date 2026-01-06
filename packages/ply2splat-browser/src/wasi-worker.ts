/**
 * WASI child worker for handling threading in the WebAssembly module.
 * This worker is spawned by the main conversion worker to handle async operations.
 * @module
 */

// IMPORTANT: Import polyfill BEFORE @napi-rs/wasm-runtime
import "./polyfill";

import {
  instantiateNapiModuleSync,
  MessageHandler,
  WASI,
  type ImportObject,
} from "@napi-rs/wasm-runtime";

const handler = new MessageHandler({
  onLoad({
    wasmModule,
    wasmMemory,
  }: {
    wasmModule: WebAssembly.Module;
    wasmMemory: WebAssembly.Memory;
  }) {
    const wasi = new WASI({
      version: "preview1",
      print(...args: unknown[]) {
        console.log.apply(console, args);
      },
      printErr(...args: unknown[]) {
        console.error.apply(console, args);
      },
    });

    return instantiateNapiModuleSync(wasmModule, {
      childThread: true,
      wasi,
      overwriteImports(importObject: ImportObject) {
        importObject.env = {
          ...importObject.env,
          ...importObject.napi,
          ...importObject.emnapi,
          memory: wasmMemory,
        };
      },
    });
  },
});

globalThis.onmessage = function (e: MessageEvent) {
  handler.handle(e);
};

