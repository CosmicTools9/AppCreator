import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    port: 49496,
    proxy: {
      "/api": {
        target: "http://localhost:49495",
        changeOrigin: true,
      },
      "/health": {
        target: "http://localhost:49495",
        changeOrigin: true,
      },
      "/auth": {
        target: "http://localhost:9002",
        changeOrigin: true,
      },
    },
  },
  build: {
    outDir: "dist",
    sourcemap: true,
  },
  // Use relative base path so built output works with file:// protocol
  // (localhost:port also serves fine with './' — modern browsers resolve correctly)
  base: "./",
});
