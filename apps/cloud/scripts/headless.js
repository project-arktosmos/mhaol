import { spawn, execSync } from 'node:child_process';
import { createServer } from 'node:net';
import { readFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = fileURLToPath(new URL('.', import.meta.url));
const envFile = readFileSync(join(__dirname, '..', '.env'), 'utf-8');
const envPort = envFile.match(/^PORT=(\d+)/m)?.[1];
const PORT = parseInt(envPort || '1510');
const BACKEND_PORT = PORT + 1; // internal only, not exposed
const HEALTH_URL = `http://localhost:${BACKEND_PORT}/api/cloud/libraries`;

// Check that port 1510 is available before doing anything
await checkPort(PORT);

const skipBuild = process.argv.includes('--skip-build');

// Build the backend first
const serverBin = join(__dirname, '..', '..', '..', 'target', 'debug', 'mhaol-server');
if (skipBuild && existsSync(serverBin)) {
	console.log('Backend already built, skipping.');
} else {
	console.log('Building backend...');
	execSync('cargo build --bin mhaol-server', { stdio: 'inherit', cwd: '../..' });
}

// Start the backend on its internal port
console.log('Starting backend on port', BACKEND_PORT);
const backend = spawn(serverBin, [], {
	stdio: 'inherit',
	env: { ...process.env, PORT: String(BACKEND_PORT) }
});

// Wait for backend to be ready
console.log('Waiting for backend...');
await waitForBackend();
console.log('Backend ready.');

// Start the frontend dev server on port 1510 with --host for LAN access
const frontend = spawn('pnpm', ['--filter', 'cloud', 'dev'], {
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

function checkPort(port) {
	return new Promise((resolve, reject) => {
		const server = createServer();
		server.once('error', (err) => {
			if (err.code === 'EADDRINUSE') {
				console.error(
					`Error: port ${port} is already in use. Stop the process using it and try again.`
				);
				process.exit(1);
			}
			reject(err);
		});
		server.once('listening', () => {
			server.close(() => resolve());
		});
		server.listen(port);
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
