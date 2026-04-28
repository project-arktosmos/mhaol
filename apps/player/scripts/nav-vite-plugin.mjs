// @ts-nocheck
import path from "node:path";
import { fileURLToPath } from "node:url";
import { generate } from "./generate-nav.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROUTES_DIR = path.resolve(__dirname, "../src/routes");

export function navPlugin() {
  return {
    name: "player-auto-nav",
    buildStart() {
      generate();
    },
    configureServer(server) {
      generate();
      server.watcher.add(ROUTES_DIR);
      const regen = (file) => {
        if (typeof file === "string" && !file.startsWith(ROUTES_DIR)) return;
        generate();
      };
      server.watcher.on("add", regen);
      server.watcher.on("unlink", regen);
      server.watcher.on("addDir", regen);
      server.watcher.on("unlinkDir", regen);
    },
  };
}
