import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

// Get the project root directory
const projectRoot = path.resolve(__dirname, "../../..");

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      "@alioth/components": path.resolve(projectRoot, "Framework/frontend/components/dist/index.es.js"),
      "@alioth/hooks": path.resolve(projectRoot, "Framework/frontend/hooks/dist/index.js"),
      "@alioth/utils": path.resolve(projectRoot, "Framework/frontend/utils/dist/index.js"),
      "@alioth/types": path.resolve(projectRoot, "Framework/frontend/types/dist/index.js"),
    },
  },
  server: {
    port: 5173,
  },
  build: {
    outDir: "dist",
    sourcemap: true,
  },
});