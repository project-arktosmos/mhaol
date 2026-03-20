import { spawn, execSync } from 'node:child_process';

const BACKEND_PORT = process.env.PORT || 1540;
const HEALTH_URL = `http://localhost:${BACKEND_PORT}/api/ytdl/status`;

// Kill any existing process on the backend port
try {
	execSync(`lsof -ti:${BACKEND_PORT} | xargs kill -9 2>/dev/null`, { stdio: 'ignore' });
} catch {
	// No process to kill
}

// Start the backend from the release build
console.log('Starting backend on port', BACKEND_PORT);
const backend = spawn('../../target/release/mhaol-server', [], {
	stdio: 'inherit',
	env: { ...process.env, PORT: String(BACKEND_PORT) }
});

// Wait for backend to be ready
console.log('Waiting for backend...');
await waitForBackend();
console.log('Backend ready.');

// Start the frontend preview server
const frontend = spawn('pnpm', ['--filter', 'video-cloud', 'preview'], {
	stdio: 'inherit',
	cwd: '../..'
});

function cleanup() {
	backend.kill();
	frontend.kill();
	process.exit();
}

process.on('SIGINT', cleanup);
process.on('SIGTERM', cleanup);
backend.on('exit', cleanup);

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
