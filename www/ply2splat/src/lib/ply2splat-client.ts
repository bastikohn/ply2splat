// Wrapper that runs ply2splat in a Web Worker
import wasmUrl from '@ply2splat/native/ply2splat-native.wasm32-wasi.wasm?url'
import wasiWorkerUrl from './wasi-worker-wrapper.js?worker&url'
import Ply2SplatWorker from './ply2splat.worker.js?worker'

export interface ConversionResult {
  data: Uint8Array
  count: number
}

let worker: Worker | null = null
let messageId = 0
const pendingMessages = new Map<number, { resolve: (value: any) => void; reject: (error: Error) => void }>()

function getWorker(): Worker {
  if (!worker) {
    console.log('[ply2splat client] Creating worker...')
    worker = new Ply2SplatWorker()
    worker.onmessage = (e) => {
      const { type, id, result, error } = e.data
      console.log('[ply2splat client] Worker message received:', type, id)
      const pending = pendingMessages.get(id)
      if (pending) {
        pendingMessages.delete(id)
        if (type === 'error') {
          pending.reject(new Error(error))
        } else {
          pending.resolve(result)
        }
      } else {
        console.warn('[ply2splat client] No pending message for id:', id)
      }
    }
    worker.onerror = (e) => {
      console.error('[ply2splat client] Worker error:', e)
    }
  }
  return worker
}

function postMessage<T>(type: string, payload?: any): Promise<T> {
  return new Promise((resolve, reject) => {
    const id = messageId++
    console.log('[ply2splat client] Posting message:', type, id)
    pendingMessages.set(id, { resolve, reject })
    getWorker().postMessage({ type, id, payload })
  })
}

let initialized = false

export async function initWasm(): Promise<void> {
  if (initialized) return
  console.log('[ply2splat client] Initializing WASM...')
  await postMessage('init', { wasmUrl, wasiWorkerUrl })
  initialized = true
  console.log('[ply2splat client] WASM initialized')
}

export async function convert(plyData: Uint8Array, sort: boolean = true): Promise<ConversionResult> {
  await initWasm()
  console.log('[ply2splat client] Converting PLY data...')
  return postMessage<ConversionResult>('convert', { plyData, sort })
}
