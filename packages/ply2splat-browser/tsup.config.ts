import { defineConfig } from "tsup";

export default defineConfig({
  entry: {
    index: "src/index.ts",
    worker: "src/worker.ts",
    "wasi-worker": "src/wasi-worker.ts",
  },
  format: ["esm"],
  dts: true,
  sourcemap: true,
  clean: true,
  splitting: false,
  treeshake: true,
  target: "es2022",
  platform: "browser",
  // Bundle these packages so the polyfill runs BEFORE @napi-rs/wasm-runtime
  // ES modules hoist imports, so if we leave @napi-rs/wasm-runtime external,
  // it will be imported before our polyfill code executes.
  noExternal: ["buffer", "@napi-rs/wasm-runtime"],
  esbuildOptions(options) {
    options.banner = {
      js: "/* @ply2splat/browser - Browser bindings for ply2splat */",
    };
  },
});
