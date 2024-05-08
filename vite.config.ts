import { defineConfig, type PluginOption } from "vite";
import { resolve } from 'path'
import react from "@vitejs/plugin-react";
import visualizer from "rollup-plugin-visualizer";
import { execSync } from "child_process";

let revision = execSync('git rev-parse --short HEAD').toString().trim();

// https://vitejs.dev/config/
export default defineConfig(async () => ({

  plugins: [react(), visualizer({ filename: "./stats/currentJSBloat_commit_" + revision + ".html", gzipSize: true, brotliSize: true }) as PluginOption],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
  },
  // 3. to make use of `TAURI_DEBUG` and other env variables
  // https://tauri.app/v1/api/config#buildconfig.beforedevcommand
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        createWettkampf: resolve(__dirname, 'createWettkampf.html'),
        editor: resolve(__dirname, 'editor.html'),
        licenses: resolve(__dirname, 'licenses.html'),
      },
    },
  },
}));