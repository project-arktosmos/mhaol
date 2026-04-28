import tailwindcss from "@tailwindcss/vite";
import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  server: {
    host: true,
    port: parseInt(process.env.PORT || "9596"),
    fs: {
      allow: ["../.."],
    },
    proxy: {
      "/api": "http://localhost:1540",
    },
  },
  preview: {
    host: true,
    port: parseInt(process.env.PORT || "9596"),
    proxy: {
      "/api": "http://localhost:1540",
    },
  },
});
