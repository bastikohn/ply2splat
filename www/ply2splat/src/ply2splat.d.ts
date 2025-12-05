declare module '../lib/ply2splat-browser.js' {
  export interface ConversionResult {
    /** The converted SPLAT data */
    data: Uint8Array
    /** Number of splats in the result */
    count: number
  }

  /**
   * Convert PLY data to SPLAT format.
   */
  export function convert(plyData: Uint8Array, sort?: boolean | null): ConversionResult

  export function getSplatCount(splatData: Uint8Array): number
  export function cli(args: string[]): void
  export function simpleFn(): number
}

declare module '@ply2splat/native/ply2splat-native.wasm32-wasi.wasm?url' {
  const url: string
  export default url
}

declare module '@ply2splat/native/wasi-worker-browser.mjs?url' {
  const url: string
  export default url
}
