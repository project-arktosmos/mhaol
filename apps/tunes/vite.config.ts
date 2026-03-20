import tailwindcss from "@tailwindcss/vite";
import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  server: {
    host: true,
    port: 1550,
    fs: {
      allow: ["../.."],
    },
    proxy: {
      "/api": "http://localhost:1550",
    },
  },
  preview: {
    host: true,
    port: 1550,
    proxy: {
      "/api": "http://localhost:1550",
    },
  },
});
