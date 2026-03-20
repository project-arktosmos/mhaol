import { defineConfig, type Plugin } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { resolve } from "path";
import { copyFileSync, mkdirSync, existsSync } from "fs";

function copyExtensionFiles(): Plugin {
  return {
    name: "copy-extension-files",
    writeBundle() {
      copyFileSync(
        resolve(__dirname, "manifest.json"),
        resolve(__dirname, "dist/manifest.json"),
      );
      const iconsDir = resolve(__dirname, "dist/icons");
      if (!existsSync(iconsDir)) mkdirSync(iconsDir, { recursive: true });
      const srcIcons = resolve(__dirname, "public/icons");
      if (existsSync(srcIcons)) {
        for (const size of ["16", "48", "128"]) {
          const file = `icon-${size}.png`;
          const src = resolve(srcIcons, file);
          if (existsSync(src)) copyFileSync(src, resolve(iconsDir, file));
        }
      }
    },
  };
}

export default defineConfig({
  plugins: [svelte(), tailwindcss(), copyExtensionFiles()],
  base: "./",
  build: {
    outDir: "dist",
    emptyOutDir: true,
    rollupOptions: {
      input: {
        popup: resolve(__dirname, "popup.html"),
        content: resolve(__dirname, "src/content/content.ts"),
        background: resolve(__dirname, "src/background/background.ts"),
      },
      output: {
        entryFileNames: "[name].js",
        chunkFileNames: "chunks/[name]-[hash].js",
        assetFileNames: "assets/[name]-[hash][extname]",
      },
    },
  },
});
