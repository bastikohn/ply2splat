/**
 * Result of a PLY to SPLAT conversion.
 */
export interface ConversionResult {
  /** The converted SPLAT data as a Uint8Array */
  data: Uint8Array;
  /** Number of splats in the result */
  count: number;
}

/**
 * Options for initializing the WASM module.
 */
export interface InitOptions {
  /**
   * URL to the WASM file. If not provided, will attempt to resolve from
   * @ply2splat/native-wasm32-wasi package.
   */
  wasmUrl?: string;
  /**
   * URL to the WASI worker file. If not provided, will use the bundled worker.
   */
  wasiWorkerUrl?: string;
  /**
   * Number of async worker threads. Default: 4
   */
  asyncWorkPoolSize?: number;
}

/**
 * Options for converting PLY to SPLAT.
 */
export interface ConvertOptions {
  /**
   * Whether to sort splats by importance. Default: true
   */
  sort?: boolean;
}

/**
 * Message types for communication between main thread and worker.
 * @internal
 */
export type WorkerMessageType = "init" | "convert" | "init-complete" | "convert-complete" | "error";

/**
 * Message sent from main thread to worker.
 * @internal
 */
export interface WorkerRequest {
  type: "init" | "convert";
  id: number;
  payload?: {
    wasmUrl?: string;
    wasiWorkerUrl?: string;
    asyncWorkPoolSize?: number;
    plyData?: Uint8Array;
    sort?: boolean;
  };
}

/**
 * Message sent from worker to main thread.
 * @internal
 */
export interface WorkerResponse {
  type: "init-complete" | "convert-complete" | "error";
  id: number;
  result?: ConversionResult;
  error?: string;
}

