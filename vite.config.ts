// import { defineConfig } from 'vite';
// import vue from '@vitejs/plugin-vue';

// const host = process.env.TAURI_DEV_HOST;

// // https://vite.dev/config/
// export default defineConfig(async () => ({
//   plugins: [vue()],
//   // Add this to externalize Tauri APIs
//   // build: {
//   //   rollupOptions: {
//   //     external: [
//   //       '@tauri-apps/api',
//   //       '@tauri-apps/api/tauri',
//   //       '@tauri-apps/plugin-dialog',
//   //       '@tauri-apps/plugin-opener',
//   //       '@tauri-apps/plugin-shell',
//   //     ],
//   //   },
//   // },
//   // Vite options tailored for Tauri development
//   clearScreen: false,
//   server: {
//     port: 1420,
//     strictPort: true,
//     host: host || false,
//     hmr: host
//       ? {
//           protocol: 'ws',
//           host,
//           port: 1421,
//         }
//       : undefined,
//     watch: {
//       // Tell Vite to ignore watching `src-tauri`
//       ignored: ['**/src-tauri/**'],
//     },
//   },
//   // Ensure Tauri-specific environment variables are included
//   envPrefix: ['VITE_', 'TAURI_'],
// }));

import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [vue()],

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
}));