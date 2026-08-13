import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import path from "path";

export default defineConfig({
  plugins: [
    tanstackRouter({
      routesDirectory: "./src/routes",
      generatedRouteTree: "./src/routeTree.gen.ts",
    }),
    react(),
    tailwindcss(),
  ],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    port: 5174,
    strictPort: true,
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    rollupOptions: {
      output: {
        manualChunks: {
          "vendor-tanstack": [
            "@tanstack/react-query",
            "@tanstack/react-router",
            "@tanstack/react-virtual",
          ],
          "vendor-ui": ["@dnd-kit/core", "@dnd-kit/sortable", "cmdk", "lucide-react"],
          "vendor-editor": ["@monaco-editor/react"],
          "vendor-graph": ["cytoscape", "dagre", "@xyflow/react"],
        },
      },
    },
  },
});
