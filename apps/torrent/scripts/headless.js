import { spawn, execSync } from 'node:child_process';
import { createServer } from 'node:net';
import { readFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = fileURLToPath(new URL('.', import.meta.url));
const ROOT = join(__dirname, '..', '..', '..');
const DIST_DIR = join(__dirname, '..', 'dist-static');

// Read port from .env
const envFile = readFileSync(join(__dirname, '..', '.env'), 'utf-8');
const envPort = envFile.match(/^PORT=(\d+)/m)?.[1];
const PORT = parseInt(envPort || '1520');

await checkPort(PORT);

const skipBuild = process.argv.includes('--skip-build');

// Build the frontend
if (skipBuild && existsSync(join(DIST_DIR, 'index.html'))) {
	console.log('Frontend already built, skipping.');
} else {
	console.log('Building frontend...');
	execSync('pnpm --filter torrent build', { stdio: 'inherit', cwd: ROOT });
}

// Build the torrent server
const torrentBin = join(ROOT, 'target', 'debug', 'mhaol-torrent-server');
if (skipBuild && existsSync(torrentBin)) {
	console.log('Torrent server already built, skipping.');
} else {
	console.log('Building torrent server...');
	execSync('cargo build -p mhaol-torrent --bin mhaol-torrent-server', {
		stdio: 'inherit',
		cwd: ROOT
	});
}

// Start the torrent server — serves both API and static frontend on a single port
console.log(`Starting torrent server on port ${PORT}...`);
const backend = spawn(torrentBin, [], {
	stdio: 'inherit',
	env: {
		...process.env,
		TORRENT_PORT: String(PORT),
		TORRENT_STATIC_DIR: DIST_DIR,
		RUST_LOG: 'info'
	}
});

// Wait for server to be ready
await waitForBackend(`http://localhost:${PORT}/api/torrent/status`);
console.log(`Torrent app running at http://localhost:${PORT}`);

function cleanup() {
	backend.kill();
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

async function waitForBackend(url) {
	while (true) {
		try {
			const res = await fetch(url);
			if (res.ok) return;
		} catch {
			// Not ready yet
		}
		await new Promise((r) => setTimeout(r, 500));
	}
}
