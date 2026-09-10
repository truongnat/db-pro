import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import path from "path";

export default defineConfig({
  base: process.env.TAURI_ENV_PLATFORM ? "./" : "/",
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
    host: "0.0.0.0",
    port: 5174,
    strictPort: true,
    // Arena and similar remote dev environments proxy Vite through a generated host.
    allowedHosts: true,
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    rollupOptions: {
      output: {
        manualChunks(id) {
          const module = id.split("node_modules/").pop() ?? "";
          if (module.startsWith("react/") || module.startsWith("react-dom/")) {
            return "vendor-react";
          }
          if (module.startsWith("@tanstack/react-")) return "vendor-tanstack";
          if (
            module.startsWith("@dnd-kit/") ||
            module.startsWith("cmdk/") ||
            module.startsWith("lucide-react/")
          ) {
            return "vendor-ui";
          }
          if (module.startsWith("@monaco-editor/")) return "vendor-editor";
          if (module.startsWith("@xyflow/")) return "vendor-reactflow";
          if (module.startsWith("cytoscape/") || module.startsWith("dagre/")) {
            return "vendor-cytoscape";
          }
        },
      },
    },
  },
});
