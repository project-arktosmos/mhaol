#!/usr/bin/env node
// Wrapper around `vite` that resolves the rendezvous bootstrap multiaddr
// and the swarm-key contents from disk and injects them as `VITE_*` env
// vars before spawning Vite. The player's browser code reads them via
// `import.meta.env` and auto-connects — no manual config UI.
//
// Resolution precedence (matches the cloud + rendezvous):
//   - swarm key:        IPFS_SWARM_KEY_FILE → ${DATA_DIR}/swarm.key →
//                       ~/mhaol/swarm.key → ~/mhaol-cloud/swarm.key
//   - bootstrap file:   RENDEZVOUS_BOOTSTRAP_FILE →
//                       ${DATA_DIR}/rendezvous/bootstrap.multiaddr →
//                       ~/mhaol/rendezvous/bootstrap.multiaddr
//   - bootstrap addrs:  RENDEZVOUS_BOOTSTRAP env var (newline- or
//                       comma-separated) wins over the file.
//
// Only `/ws` / `/wss` / `/webtransport` multiaddrs are kept — browsers
// can't speak raw TCP, so the plain TCP entries the rendezvous also
// writes would be useless to the player.

import { spawn } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const argv = process.argv.slice(2);

function homeDir() {
	return os.homedir() || process.env.HOME || process.env.USERPROFILE || '';
}

function dataDir() {
	if (process.env.DATA_DIR) return process.env.DATA_DIR;
	const home = homeDir();
	return home ? path.join(home, 'mhaol') : '';
}

function firstReadable(...candidates) {
	for (const c of candidates) {
		if (!c) continue;
		try {
			fs.accessSync(c, fs.constants.R_OK);
			return c;
		} catch {
			// try next
		}
	}
	return null;
}

function readSwarmKey() {
	const home = homeDir();
	const dd = dataDir();
	const candidate = firstReadable(
		process.env.IPFS_SWARM_KEY_FILE,
		dd ? path.join(dd, 'swarm.key') : null,
		home ? path.join(home, 'mhaol', 'swarm.key') : null,
		home ? path.join(home, 'mhaol-cloud', 'swarm.key') : null
	);
	if (!candidate) return null;
	try {
		const txt = fs.readFileSync(candidate, 'utf8');
		if (!txt.startsWith('/key/swarm/psk/1.0.0/')) {
			console.warn(
				`[player] swarm key at ${candidate} does not start with /key/swarm/psk/1.0.0/ — ignoring`
			);
			return null;
		}
		return { path: candidate, contents: txt };
	} catch (err) {
		console.warn(`[player] failed to read swarm key at ${candidate}:`, err.message);
		return null;
	}
}

function readBootstrap() {
	const fromEnv = process.env.RENDEZVOUS_BOOTSTRAP;
	const lines = [];
	if (fromEnv) {
		for (const line of fromEnv.split(/[\n,]/)) {
			const trimmed = line.trim();
			if (trimmed) lines.push(trimmed);
		}
	} else {
		const home = homeDir();
		const dd = dataDir();
		const candidate = firstReadable(
			process.env.RENDEZVOUS_BOOTSTRAP_FILE,
			dd ? path.join(dd, 'rendezvous', 'bootstrap.multiaddr') : null,
			home ? path.join(home, 'mhaol', 'rendezvous', 'bootstrap.multiaddr') : null
		);
		if (!candidate) return { source: null, lines: [] };
		try {
			const txt = fs.readFileSync(candidate, 'utf8');
			for (const line of txt.split(/\r?\n/)) {
				const trimmed = line.trim();
				if (trimmed) lines.push(trimmed);
			}
			return { source: candidate, lines: filterBrowserDialable(lines) };
		} catch (err) {
			console.warn(`[player] failed to read bootstrap file at ${candidate}:`, err.message);
			return { source: null, lines: [] };
		}
	}
	return { source: 'env:RENDEZVOUS_BOOTSTRAP', lines: filterBrowserDialable(lines) };
}

function filterBrowserDialable(lines) {
	return lines.filter(
		(addr) =>
			addr.includes('/ws/') ||
			addr.endsWith('/ws') ||
			addr.includes('/wss/') ||
			addr.endsWith('/wss') ||
			addr.includes('/webtransport')
	);
}

const swarm = readSwarmKey();
const bootstrap = readBootstrap();

const env = { ...process.env };

if (swarm) {
	env.VITE_SWARM_KEY = swarm.contents;
	console.log(`[player] swarm key loaded from ${swarm.path}`);
} else {
	console.warn(
		'[player] no swarm.key found — start `pnpm app:rendezvous` first, or set IPFS_SWARM_KEY_FILE'
	);
}

if (bootstrap.lines.length > 0) {
	env.VITE_RENDEZVOUS_BOOTSTRAP = bootstrap.lines.join('\n');
	console.log(
		`[player] bootstrap addrs (${bootstrap.lines.length}) loaded from ${bootstrap.source}`
	);
	for (const line of bootstrap.lines) console.log(`  ${line}`);
} else {
	console.warn(
		'[player] no browser-dialable (/ws | /wss | /webtransport) bootstrap addrs found — start `pnpm app:rendezvous` so it writes ${DATA_DIR}/rendezvous/bootstrap.multiaddr, or set RENDEZVOUS_BOOTSTRAP'
	);
}

// Resolve vite from the local install so this works whether the script
// is invoked via pnpm (`node_modules/.bin` is on PATH) or directly
// (`node scripts/run-vite.mjs`, where it isn't).
function resolveVite() {
	const isWindows = process.platform === 'win32';
	const binName = isWindows ? 'vite.cmd' : 'vite';
	let dir = path.resolve(__dirname, '..');
	while (true) {
		const candidate = path.join(dir, 'node_modules', '.bin', binName);
		if (fs.existsSync(candidate)) return candidate;
		const parent = path.dirname(dir);
		if (parent === dir) break;
		dir = parent;
	}
	return binName;
}

const child = spawn(resolveVite(), argv, { env, stdio: 'inherit', shell: false });
child.on('exit', (code, signal) => {
	if (signal) process.kill(process.pid, signal);
	else process.exit(code ?? 0);
});
