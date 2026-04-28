import tailwindcss from "@tailwindcss/vite";
import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  server: {
    host: true,
    port: parseInt(process.env.PORT || "9595"),
    fs: {
      allow: ["../.."],
    },
    proxy: {
      "/api": "http://localhost:1530",
    },
  },
  preview: {
    host: true,
    port: parseInt(process.env.PORT || "9595"),
    proxy: {
      "/api": "http://localhost:1530",
    },
  },
});
