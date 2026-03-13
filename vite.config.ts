import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from '@tailwindcss/vite'

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
        protocol: "ws",
        host,
        port: 1421,
      }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    // Target modern Chromium (WebView2) — outputs smaller, faster ES2020 code
    target: "es2020",
    // Raise Rollup's warning threshold — our vendor chunks are intentionally large
    chunkSizeWarningLimit: 600,
    rollupOptions: {
      output: {
        // Split stable vendor libs into separate cacheable chunks
        manualChunks: {
          "react-vendor": ["react", "react-dom"],
          "charts": ["recharts"],
          "utils": ["date-fns"],
        },
      },
    },
  },
}));
