// Wrapper for wasi-worker that adds Buffer polyfill

// IMPORTANT: Import polyfill BEFORE @napi-rs/wasm-runtime
import "./worker-polyfill.js";

import { instantiateNapiModuleSync, MessageHandler, WASI } from "@napi-rs/wasm-runtime";

const handler = new MessageHandler({
  onLoad({ wasmModule, wasmMemory }) {
    const wasi = new WASI({
      print: function () {
        console.log.apply(console, arguments);
      },
      printErr: function () {
        console.error.apply(console, arguments);
      },
    });
    return instantiateNapiModuleSync(wasmModule, {
      childThread: true,
      wasi,
      overwriteImports(importObject) {
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

globalThis.onmessage = function (e) {
  handler.handle(e);
};
