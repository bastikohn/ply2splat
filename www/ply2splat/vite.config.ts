import { defineConfig, type Plugin } from 'vite'
import { devtools } from '@tanstack/devtools-vite'
import viteReact from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

import { tanstackRouter } from '@tanstack/router-plugin/vite'
import { fileURLToPath, URL } from 'node:url'

const napiDir = fileURLToPath(new URL('../../crates/ply2splat-napi', import.meta.url))

// Plugin to ensure COOP/COEP headers are set for SharedArrayBuffer support
function crossOriginIsolation(): Plugin {
  return {
    name: 'cross-origin-isolation',
    configureServer(server) {
      server.middlewares.use((_req, res, next) => {
        res.setHeader('Cross-Origin-Opener-Policy', 'same-origin')
        res.setHeader('Cross-Origin-Embedder-Policy', 'require-corp')
        next()
      })
    },
    configurePreviewServer(server) {
      server.middlewares.use((_req, res, next) => {
        res.setHeader('Cross-Origin-Opener-Policy', 'same-origin')
        res.setHeader('Cross-Origin-Embedder-Policy', 'require-corp')
        next()
      })
    },
  }
}

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    crossOriginIsolation(),
    devtools(),
    tanstackRouter({
      target: 'react',
      autoCodeSplitting: true,
    }),
    viteReact({
      babel: {
        plugins: ['babel-plugin-react-compiler'],
      },
    }),
    tailwindcss(),
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
      // Alias files from @ply2splat/native for browser usage
      '@ply2splat/native/ply2splat-native.wasm32-wasi.wasm': `${napiDir}/ply2splat-native.wasm32-wasi.wasm`,
      '@ply2splat/native/wasi-worker-browser.mjs': `${napiDir}/wasi-worker-browser.mjs`,
    },
  },
  server: {
    fs: {
      // Allow serving files from the napi crate directory
      allow: ['.', napiDir],
    },
  },
  worker: {
    format: 'es',
  },
  optimizeDeps: {
    exclude: ['@ply2splat/native', '@napi-rs/wasm-runtime'],
  },
  build: {
    target: 'esnext',
  },
  assetsInclude: ['**/*.wasm'],
})
