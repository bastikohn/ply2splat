/**
 * Ambient type declarations for @napi-rs/wasm-runtime
 */

declare module "@napi-rs/wasm-runtime" {
  export interface WASIOptions {
    version: "preview1";
    print?(...args: unknown[]): void;
    printErr?(...args: unknown[]): void;
  }

  export class WASI {
    constructor(options: WASIOptions);
  }

  export interface EmnapiContext {
    // opaque context
  }

  export function getDefaultContext(): EmnapiContext;

  export interface ImportObject {
    env?: Record<string, unknown>;
    napi?: Record<string, unknown>;
    emnapi?: Record<string, unknown>;
    [key: string]: unknown;
  }

  export interface NapiModuleOptions {
    context: EmnapiContext;
    asyncWorkPoolSize?: number;
    wasi: WASI;
    onCreateWorker?: () => Worker;
    overwriteImports?: (importObject: ImportObject) => ImportObject;
    beforeInit?: (params: { instance: WebAssembly.Instance }) => void;
    childThread?: boolean;
  }

  export interface NapiModuleSyncOptions {
    childThread?: boolean;
    wasi: WASI;
    overwriteImports?: (importObject: ImportObject) => void;
  }

  export interface NapiModule {
    exports: Record<string, unknown>;
  }

  export interface InstantiateResult {
    instance: WebAssembly.Instance;
    module: WebAssembly.Module;
    napiModule: NapiModule;
  }

  export function instantiateNapiModule(
    wasmFile: ArrayBuffer,
    options: NapiModuleOptions
  ): Promise<InstantiateResult>;

  export function instantiateNapiModuleSync(
    wasmModule: WebAssembly.Module,
    options: NapiModuleSyncOptions
  ): InstantiateResult;

  export interface MessageHandlerOptions {
    onLoad(params: {
      wasmModule: WebAssembly.Module;
      wasmMemory: WebAssembly.Memory;
    }): InstantiateResult;
  }

  export class MessageHandler {
    constructor(options: MessageHandlerOptions);
    handle(event: MessageEvent): void;
  }
}

// Let TS accept bundler "URL imports" like `*.wasm?url` / `./worker?worker&url`
declare module "*?url" {
  const url: string;
  export default url;
}

// (optional) if you still use these anywhere:
declare module "*?worker&url" {
  const url: string;
  export default url;
}
