import { spawn, execSync } from "node:child_process";

const BACKEND_PORT = process.env.PORT || 1550;
const HEALTH_URL = `http://localhost:${BACKEND_PORT}/api/health`;

// Kill any existing process on the backend port
try {
  execSync(`lsof -ti:${BACKEND_PORT} | xargs kill -9 2>/dev/null`, {
    stdio: "ignore",
  });
} catch {
  // No process to kill
}

// Load env vars from .env.app if it exists
try {
  execSync("set -a; . ../../.env.app 2>/dev/null; set +a", { stdio: "ignore" });
} catch {
  // No .env.app
}

// Build the backend first
console.log("Building backend...");
execSync("cargo build --bin mhaol-server", { stdio: "inherit", cwd: "../.." });

// Start the backend
console.log("Starting backend on port", BACKEND_PORT);
const backend = spawn(
  "cargo",
  ["run", "-p", "mhaol-backend", "--bin", "mhaol-server"],
  {
    stdio: "inherit",
    cwd: "../..",
    env: { ...process.env, PORT: String(BACKEND_PORT) },
  },
);

// Wait for backend to be ready
console.log("Waiting for backend...");
await waitForBackend();
console.log("Backend ready.");

// Start the frontend dev server
const frontend = spawn("pnpm", ["--filter", "tunes", "dev"], {
  stdio: "inherit",
  cwd: "../..",
});

function cleanup() {
  backend.kill();
  frontend.kill();
  process.exit();
}

process.on("SIGINT", cleanup);
process.on("SIGTERM", cleanup);
backend.on("exit", cleanup);

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
