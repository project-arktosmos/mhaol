import { spawn, execSync } from 'node:child_process';
import { createServer as createNetServer } from 'node:net';
import { createServer, request as httpRequest } from 'node:http';
import { createReadStream, readFileSync, existsSync, statSync } from 'node:fs';
import { join, extname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { networkInterfaces } from 'node:os';

const __dirname = fileURLToPath(new URL('.', import.meta.url));
const envFile = readFileSync(join(__dirname, '..', '.env'), 'utf-8');
const envPort = envFile.match(/^PORT=(\d+)/m)?.[1];
const PORT = parseInt(envPort || '1520');
const BACKEND_PORT = PORT + 1; // internal only, not exposed
const HEALTH_URL = `http://localhost:${BACKEND_PORT}/api/signaling/status`;
const STATIC_DIR = join(__dirname, '..', 'dist-static');

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

await checkPort(PORT);

// Build frontend
console.log('Building frontend...');
execSync('pnpm --filter signaling-app build', { stdio: 'inherit', cwd: '../..' });

// Build + start backend on internal port
console.log('Building backend...');
execSync('cargo build --bin mhaol-server', { stdio: 'inherit', cwd: '../..' });

console.log('Starting backend on internal port', BACKEND_PORT);
const backend = spawn('cargo', ['run', '-p', 'mhaol-backend', '--bin', 'mhaol-server'], {
	stdio: 'inherit',
	cwd: '../..',
	env: { ...process.env, PORT: String(BACKEND_PORT) }
});

console.log('Waiting for backend...');
await waitForBackend();
console.log('Backend ready.');

// Single server on PORT: static files + proxy to backend
const server = createServer((req, res) => {
	const url = req.url || '/';

	// Proxy /api and /party to backend
	if (url.startsWith('/api') || url.startsWith('/party')) {
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
	let filePath = join(STATIC_DIR, url === '/' ? 'index.html' : url);
	if (!existsSync(filePath) || statSync(filePath).isDirectory()) {
		filePath = join(STATIC_DIR, 'index.html'); // SPA fallback
	}

	const ext = extname(filePath);
	const contentType = MIME_TYPES[ext] || 'application/octet-stream';

	const stream = createReadStream(filePath);
	stream.on('error', () => {
		res.writeHead(404);
		res.end('Not found');
	});
	res.writeHead(200, { 'Content-Type': contentType });
	stream.pipe(res);
});

// WebSocket upgrade proxy for /party
server.on('upgrade', (req, socket, head) => {
	const url = req.url || '';
	if (url.startsWith('/party')) {
		const proxyReq = httpRequest({
			hostname: 'localhost',
			port: BACKEND_PORT,
			path: url,
			method: req.method,
			headers: req.headers
		});

		proxyReq.on('upgrade', (proxyRes, proxySocket, proxyHead) => {
			socket.write(
				`HTTP/1.1 ${proxyRes.statusCode || 101} ${proxyRes.statusMessage || 'Switching Protocols'}\r\n` +
					Object.entries(proxyRes.headers)
						.map(([k, v]) => `${k}: ${v}`)
						.join('\r\n') +
					'\r\n\r\n'
			);
			if (proxyHead.length) socket.write(proxyHead);
			proxySocket.pipe(socket);
			socket.pipe(proxySocket);
		});

		proxyReq.on('error', () => socket.destroy());
		proxyReq.end();
	} else {
		socket.destroy();
	}
});

server.listen(PORT, '0.0.0.0', () => {
	const lanIp = getLanIp();
	console.log(`Signaling app running on:`);
	console.log(`  Local:   http://localhost:${PORT}`);
	if (lanIp) console.log(`  Network: http://${lanIp}:${PORT}`);
});

function getLanIp() {
	const nets = networkInterfaces();
	for (const ifaces of Object.values(nets)) {
		for (const iface of ifaces || []) {
			if (iface.family === 'IPv4' && !iface.internal) return iface.address;
		}
	}
	return null;
}

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
		const srv = createNetServer();
		srv.once('error', (err) => {
			if (err.code === 'EADDRINUSE') {
				console.error(
					`Error: port ${port} is already in use. Stop the process using it and try again.`
				);
				process.exit(1);
			}
			reject(err);
		});
		srv.once('listening', () => {
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
