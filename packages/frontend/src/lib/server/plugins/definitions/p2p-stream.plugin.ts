import { WorkerBridge } from '$lib/server/worker-bridge';
import { existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { PluginCompanion } from '../types';

const __dirname = dirname(fileURLToPath(import.meta.url));
const PACKAGE_ROOT = join(__dirname, '..', '..', '..', '..');

let workerBridge: WorkerBridge | null = null;

function resolveBinaryPath(): string | null {
	const envPath = process.env.P2P_STREAM_BIN;
	if (envPath && existsSync(envPath)) return envPath;

	const defaultPath = join(PACKAGE_ROOT, '..', 'p2p-stream', 'target', 'debug', 'p2p-stream-worker');
	if (existsSync(defaultPath)) return defaultPath;

	return null;
}

export const p2pStreamCompanion: PluginCompanion = {
	locals: {
		p2pWorkerBridge: () => workerBridge,
		streamServerAvailable: () => workerBridge?.isAvailable() ?? false
	},

	onInit: async () => {
		const binaryPath = resolveBinaryPath();
		if (!binaryPath) {
			console.warn('[p2p-stream] Binary not found, worker disabled');
			return;
		}

		workerBridge = new WorkerBridge('p2p-stream');
		workerBridge.start(binaryPath, {
			RUST_LOG: process.env.RUST_LOG ?? 'info'
		});
	},

	onShutdown: () => {
		workerBridge?.shutdown();
		workerBridge = null;
	}
};
