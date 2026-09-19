import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { fileURLToPath, URL } from "node:url";

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [vue({
      template: {
        compilerOptions: {
          // Treat all tags starting with 'ion-' as custom elements
          isCustomElement: (tag) => tag.startsWith('onyks-')
        }
      }
    })],
    base: '',
    resolve: {
      alias: {
        // Code ported from the Bloodstone web GUI (src/web) imports through this alias.
        "@web": fileURLToPath(new URL("./src/web", import.meta.url)),
      },
    },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
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
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
