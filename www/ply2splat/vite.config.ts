import { defineConfig, type Plugin } from "vite";
import { devtools } from "@tanstack/devtools-vite";
import viteReact from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

import { tanstackRouter } from "@tanstack/router-plugin/vite";
import { fileURLToPath, URL } from "node:url";

// Plugin to ensure COOP/COEP headers are set for SharedArrayBuffer support
function crossOriginIsolation(): Plugin {
  return {
    name: "cross-origin-isolation",
    configureServer(server) {
      server.middlewares.use((_req, res, next) => {
        res.setHeader("Cross-Origin-Opener-Policy", "same-origin");
        res.setHeader("Cross-Origin-Embedder-Policy", "require-corp");
        next();
      });
    },
    configurePreviewServer(server) {
      server.middlewares.use((_req, res, next) => {
        res.setHeader("Cross-Origin-Opener-Policy", "same-origin");
        res.setHeader("Cross-Origin-Embedder-Policy", "require-corp");
        next();
      });
    },
  };
}

// Plugin to serve splat preview data through HTTP endpoints
// This avoids blob: URL issues in cross-origin isolated contexts
function splatPreviewServer(): Plugin {
  const splatStore = new Map<string, Buffer>();

  return {
    name: "splat-preview-server",
    configureServer(server) {
      // POST endpoint to store splat data
      server.middlewares.use("/__splat_preview", (req, res, next) => {
        if (req.method === "POST") {
          const chunks: Buffer[] = [];
          req.on("data", (chunk) => chunks.push(chunk));
          req.on("end", () => {
            const id = crypto.randomUUID();
            const data = Buffer.concat(chunks);
            splatStore.set(id, data);
            // Auto-cleanup after 5 minutes
            setTimeout(() => splatStore.delete(id), 5 * 60 * 1000);
            res.setHeader("Content-Type", "application/json");
            res.end(JSON.stringify({ id }));
          });
        } else if (req.method === "GET") {
          const url = new URL(req.url || "", `http://${req.headers.host}`);
          const id = url.searchParams.get("id");
          if (id && splatStore.has(id)) {
            const data = splatStore.get(id)!;
            res.setHeader("Content-Type", "application/octet-stream");
            res.setHeader("Content-Length", data.length.toString());
            res.setHeader("Cross-Origin-Resource-Policy", "same-origin");
            res.end(data);
          } else {
            res.statusCode = 404;
            res.end("Not found");
          }
        } else {
          next();
        }
      });
    },
  };
}

// https://vitejs.dev/config/
export default defineConfig({
  // Set base path for GitHub Pages deployment
  // - When VITE_BASE_PATH is set (e.g., /ply2splat/): Uses that path for GitHub Pages
  // - When not set (local dev): Uses root path (/) for localhost
  base: process.env.VITE_BASE_PATH || "/",
  plugins: [
    crossOriginIsolation(),
    splatPreviewServer(),
    devtools(),
    tanstackRouter({
      target: "react",
      autoCodeSplitting: true,
    }),
    viteReact({
      babel: {
        plugins: ["babel-plugin-react-compiler"],
      },
    }),
    tailwindcss(),
  ],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  worker: {
    format: "es",
  },
  optimizeDeps: {
    exclude: [
      "@ply2splat/native",
      "@ply2splat/native-wasm32-wasi",
      "@napi-rs/wasm-runtime",
    ],
  },
  build: {
    target: "esnext",
  },
  assetsInclude: ["**/*.wasm"],
});
