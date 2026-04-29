import tailwindcss from "@tailwindcss/vite";
import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import { navPlugin } from "./scripts/nav-vite-plugin.mjs";

const proxy = {
  "/api/documents": "http://127.0.0.1:9899",
  "/api/torrent/list": "http://127.0.0.1:9899",
  "/api/torrent/add": "http://127.0.0.1:9899",
  "/api/p2p-stream": "http://127.0.0.1:9899",
  "/api/ytdl": "http://127.0.0.1:9897",
  "/api": "http://localhost:1530",
};

export default defineConfig({
  plugins: [navPlugin(), tailwindcss(), sveltekit()],
  server: {
    host: true,
    port: parseInt(process.env.PORT || "9595"),
    fs: {
      allow: ["../.."],
    },
    proxy,
  },
  preview: {
    host: true,
    port: parseInt(process.env.PORT || "9595"),
    proxy,
  },
});
