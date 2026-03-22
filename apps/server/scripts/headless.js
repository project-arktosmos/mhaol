import { spawn, execSync } from "node:child_process";
import { createServer as createNetServer } from "node:net";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { networkInterfaces, homedir } from "node:os";

const __dirname = fileURLToPath(new URL(".", import.meta.url));
const PORT = 1530;
const CLIENT_PORT = 1570;
const DATA_DIR = join(homedir(), "mhaol-server");
const HEALTH_URL = `http://localhost:${PORT}/api/health`;
const STATIC_DIR = join(__dirname, "..", "dist-static");
const CLIENT_STATIC_DIR = join(__dirname, "..", "..", "client", "dist-static");

await checkPort(PORT);

const skipBuild = process.argv.includes("--skip-build");

// Build frontend
if (skipBuild && existsSync(join(STATIC_DIR, "index.html"))) {
  console.log("Frontend already built, skipping.");
} else {
  console.log("Building frontend...");
  execSync("pnpm --filter server build", { stdio: "inherit", cwd: "../.." });
}

// Build client app
if (skipBuild && existsSync(join(CLIENT_STATIC_DIR, "index.html"))) {
  console.log("Client app already built, skipping.");
} else {
  console.log("Building client app...");
  execSync("pnpm --filter client build", { stdio: "inherit", cwd: "../.." });
}

// Build backend
const serverBin = join(
  __dirname,
  "..",
  "..",
  "..",
  "target",
  "debug",
  "mhaol-server",
);
if (skipBuild && existsSync(serverBin)) {
  console.log("Backend already built, skipping.");
} else {
  console.log("Building backend...");
  execSync("cargo build -p mhaol-server --bin mhaol-server", {
    stdio: "inherit",
    cwd: "../..",
  });
}

// Start backend on PORT with STATIC_DIR so it serves both API and static files
console.log(`Starting backend on port ${PORT}`);
const backend = spawn(serverBin, [], {
  stdio: "inherit",
  env: {
    ...process.env,
    PORT: String(PORT),
    CLIENT_PORT: String(CLIENT_PORT),
    STATIC_DIR,
    CLIENT_STATIC_DIR,
    DATA_DIR,
    APP_ID: "server",
  },
});

console.log("Waiting for backend...");
await waitForBackend();

const lanIp = getLanIp();
console.log(`Server app running on:`);
console.log(`  Server:  http://localhost:${PORT}`);
console.log(`  Client:  http://localhost:${CLIENT_PORT}`);
if (lanIp) {
  console.log(`  Network: http://${lanIp}:${PORT}`);
  console.log(`  Network: http://${lanIp}:${CLIENT_PORT} (client)`);
}

function getLanIp() {
  const nets = networkInterfaces();
  for (const ifaces of Object.values(nets)) {
    for (const iface of ifaces || []) {
      if (iface.family === "IPv4" && !iface.internal) return iface.address;
    }
  }
  return null;
}

function cleanup() {
  backend.kill();
  process.exit();
}

process.on("SIGINT", cleanup);
process.on("SIGTERM", cleanup);
backend.on("exit", cleanup);

function checkPort(port) {
  return new Promise((resolve, reject) => {
    const srv = createNetServer();
    srv.once("error", (err) => {
      if (err.code === "EADDRINUSE") {
        console.error(
          `Error: port ${port} is already in use. Stop the process using it and try again.`,
        );
        process.exit(1);
      }
      reject(err);
    });
    srv.once("listening", () => {
      srv.close(() => resolve());
    });
    srv.listen(port);
  });
}

async function waitForBackend() {
  while (true) {
    try {
      const res = await fetch(HEALTH_URL);
      if (res.ok) return;
    } catch {
      // Not ready yet
    }
    await new Promise((r) => setTimeout(r, 500));
  }
}
