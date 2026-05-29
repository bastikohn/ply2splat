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
 * Message sent from main thread to worker.
 * @internal
 */
export type WorkerRequest =
  | {
      type: "init";
      id: number;
      payload: {
        wasmUrl: string;
        wasiWorkerUrl: string;
        asyncWorkPoolSize: number;
      };
    }
  | {
      type: "convert";
      id: number;
      payload: {
        plyData: Uint8Array;
        sort: boolean;
      };
    };

/**
 * Message sent from worker to main thread.
 * @internal
 */
export type WorkerResponse =
  | {
      type: "init-complete";
      id: number;
    }
  | {
      type: "convert-complete";
      id: number;
      result: ConversionResult;
    }
  | {
      type: "error";
      id: number;
      error: string;
    };
