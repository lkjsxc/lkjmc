import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
export default defineConfig({
  plugins: [react()],
  cacheDir: "../.local/vite",
  server: {
    proxy: {
      "/api": "http://127.0.0.1:18091",
      "/auth": "http://127.0.0.1:18091",
      "/health": "http://127.0.0.1:18091",
    },
  },
  build: { sourcemap: true },
});
