import adapterStatic from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit') .Config} */
const config = {
  preprocess: vitePreprocess(),

  kit: {
    adapter: adapterStatic({
      fallback: "index.html",
      pages: "dist-static",
      assets: "dist-static",
    }),
    alias: {
      $components: "../../packages/ui-lib/src/components",
      $utils: "../../packages/frontend/src/utils",
      $types: "../../packages/frontend/src/types",
      $data: "../../packages/frontend/src/data",
      $adapters: "../../packages/frontend/src/adapters",
      $services: "../../packages/frontend/src/services",
      frontend: "../../packages/frontend/src",
      "ui-lib": "../../packages/ui-lib/src",
      "torrent-search-thepiratebay":
        "../../packages/addons/torrent-search-thepiratebay/src/index.ts",
    },
  },
};

export default config;
