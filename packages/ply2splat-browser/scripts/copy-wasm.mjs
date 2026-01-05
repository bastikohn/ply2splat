import fs from "node:fs";
import path from "node:path";

const pkgRoot = path.resolve(process.cwd());
const distDir = path.join(pkgRoot, "dist");

function assertExists(filePath) {
  if (!fs.existsSync(filePath)) {
    throw new Error(`File not found: ${filePath}`);
  }
}

function ensureDir(dirPath) {
  fs.mkdirSync(dirPath, { recursive: true });
}

function copyFile(src, dest) {
  ensureDir(path.dirname(dest));
  fs.copyFileSync(src, dest);
  console.log(`[ply2splat-browser] Copied ${path.basename(src)} -> dist/`);
}

// Order matters:
// 1) Prefer installed dependency path (works in published installs)
// 2) Fallback to monorepo crate path (works during local dev in this repo)
const candidates = [
  path.join(
    pkgRoot,
    "node_modules",
    "@ply2splat",
    "native-wasm32-wasi",
    "ply2splat-native.wasm32-wasi.wasm"
  ),
  path.resolve(
    pkgRoot,
    "..",
    "..",
    "crates",
    "ply2splat-napi",
    "ply2splat-native.wasm32-wasi.wasm"
  ),
];

const wasmSrc = candidates.find((p) => fs.existsSync(p));
if (!wasmSrc) {
  throw new Error(
    [
      "Could not locate ply2splat-native.wasm32-wasi.wasm.",
      "Tried:",
      ...candidates.map((p) => `- ${p}`),
    ].join("\n")
  );
}

assertExists(distDir);
copyFile(wasmSrc, path.join(distDir, "ply2splat-native.wasm32-wasi.wasm"));