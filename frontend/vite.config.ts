import react from "@vitejs/plugin-react";
import fs from "fs/promises";
import { defineConfig } from "vite";

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => ({
  plugins: [
    react(),
    {
      name: "index-html-env",
      transformIndexHtml: {
        order: "pre",
        async handler() {
          if (mode !== "production") {
            return await fs.readFile("index.dev.html", "utf8");
          }
        },
      },
    },
  ],
  server: {
    proxy: {
      "/api": "http://127.0.0.1:3000",
    },
  },
}));
