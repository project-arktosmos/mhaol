import { spawn, execSync } from 'node:child_process';
import { createServer as createNetServer } from 'node:net';
import { createServer, request as httpRequest } from 'node:http';
import { readFileSync, existsSync, statSync } from 'node:fs';
import { join, extname } from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = fileURLToPath(new URL('.', import.meta.url));

// Read port from .env
const envFile = readFileSync(join(__dirname, '..', '.env'), 'utf-8');
const envPort = envFile.match(/^PORT=(\d+)/m)?.[1];
const PORT = parseInt(envPort || '1530');
const BACKEND_PORT = PORT - 1;
const HEALTH_URL = `http://localhost:${BACKEND_PORT}/api/torrent/status`;
const DIST_DIR = join(__dirname, '..', 'dist-static');

const MIME_TYPES = {
	'.html': 'text/html',
	'.js': 'application/javascript',
	'.css': 'text/css',
	'.json': 'application/json',
	'.png': 'image/png',
	'.svg': 'image/svg+xml',
	'.ico': 'image/x-icon',
	'.woff': 'font/woff',
	'.woff2': 'font/woff2'
};

// Check that port 1530 is available before doing anything
await checkPort(PORT);

// Build the frontend first
console.log('Building frontend...');
execSync('pnpm --filter torrent build', {
	stdio: 'inherit',
	cwd: join(__dirname, '..', '..', '..')
});

// Build the backend
console.log('Building backend...');
execSync('cargo build --bin mhaol-server', {
	stdio: 'inherit',
	cwd: join(__dirname, '..', '..', '..')
});

// Start the backend on its internal port
console.log('Starting backend on port', BACKEND_PORT);
const backend = spawn('cargo', ['run', '-p', 'mhaol-backend', '--bin', 'mhaol-server'], {
	stdio: 'inherit',
	cwd: join(__dirname, '..', '..', '..'),
	env: { ...process.env, PORT: String(BACKEND_PORT) }
});

// Wait for backend to be ready
console.log('Waiting for backend...');
await waitForBackend();
console.log('Backend ready.');

// Start HTTP server on port 1530 that serves static files and proxies /api to backend
const server = createServer((req, res) => {
	const url = req.url || '/';

	// Proxy /api requests to the backend
	if (url.startsWith('/api')) {
		const proxyReq = httpRequest(
			{
				hostname: 'localhost',
				port: BACKEND_PORT,
				path: url,
				method: req.method,
				headers: req.headers
			},
			(proxyRes) => {
				res.writeHead(proxyRes.statusCode, proxyRes.headers);
				proxyRes.pipe(res);
			}
		);
		proxyReq.on('error', () => {
			res.writeHead(502);
			res.end('Backend unavailable');
		});
		req.pipe(proxyReq);
		return;
	}

	// Serve static files
	let filePath = join(DIST_DIR, url === '/' ? 'index.html' : url);

	// If path is a directory, try index.html inside it
	if (existsSync(filePath) && statSync(filePath).isDirectory()) {
		filePath = join(filePath, 'index.html');
	}

	if (existsSync(filePath) && statSync(filePath).isFile()) {
		const ext = extname(filePath);
		const contentType = MIME_TYPES[ext] || 'application/octet-stream';

		// Immutable assets get long cache headers
		const cacheControl = filePath.includes('/immutable/')
			? 'public, max-age=31536000, immutable'
			: 'public, max-age=60';

		res.writeHead(200, { 'Content-Type': contentType, 'Cache-Control': cacheControl });
		res.end(readFileSync(filePath));
	} else {
		// SPA fallback: serve index.html for client-side routing
		const indexPath = join(DIST_DIR, 'index.html');
		if (existsSync(indexPath)) {
			res.writeHead(200, { 'Content-Type': 'text/html', 'Cache-Control': 'public, max-age=60' });
			res.end(readFileSync(indexPath));
		} else {
			res.writeHead(404);
			res.end('Not found');
		}
	}
});

server.listen(PORT, '0.0.0.0', () => {
	console.log(`Torrent app running at http://localhost:${PORT}`);
});

function cleanup() {
	backend.kill();
	server.close();
	process.exit();
}

process.on('SIGINT', cleanup);
process.on('SIGTERM', cleanup);
backend.on('exit', cleanup);

function checkPort(port) {
	return new Promise((resolve, reject) => {
		const server = createNetServer();
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
