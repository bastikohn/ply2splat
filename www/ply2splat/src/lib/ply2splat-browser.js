import {
  getDefaultContext as __emnapiGetDefaultContext,
  instantiateNapiModule as __emnapiInstantiateNapiModule,
  WASI as __WASI,
} from '@napi-rs/wasm-runtime'

const __wasi = new __WASI({
  version: 'preview1',
})

// Use static imports for Vite to properly handle these assets
import wasmUrl from '@ply2splat/native/ply2splat-native.wasm32-wasi.wasm?url'
import workerUrl from '@ply2splat/native/wasi-worker-browser.mjs?url'

const __emnapiContext = __emnapiGetDefaultContext()

const __sharedMemory = new WebAssembly.Memory({
  initial: 4000,
  maximum: 65536,
  shared: true,
})

const __wasmFile = await fetch(wasmUrl).then((res) => res.arrayBuffer())

// Use async version to avoid Atomics.wait on main thread
const {
  instance: __napiInstance,
  module: __wasiModule,
  napiModule: __napiModule,
} = await __emnapiInstantiateNapiModule(__wasmFile, {
  context: __emnapiContext,
  asyncWorkPoolSize: 4,
  wasi: __wasi,
  onCreateWorker() {
    const worker = new Worker(workerUrl, {
      type: 'module',
    })
    return worker
  },
  overwriteImports(importObject) {
    importObject.env = {
      ...importObject.env,
      ...importObject.napi,
      ...importObject.emnapi,
      memory: __sharedMemory,
    }
    return importObject
  },
  beforeInit({ instance }) {
    for (const name of Object.keys(instance.exports)) {
      if (name.startsWith('__napi_register__')) {
        instance.exports[name]()
      }
    }
  },
})

export default __napiModule.exports
export const cli = __napiModule.exports.cli
export const convert = __napiModule.exports.convert
export const getSplatCount = __napiModule.exports.getSplatCount
export const simpleFn = __napiModule.exports.simpleFn
