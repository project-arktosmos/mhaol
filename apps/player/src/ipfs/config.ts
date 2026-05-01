/**
 * Build-time IPFS configuration baked in by `scripts/run-vite.mjs`. The
 * wrapper reads the swarm key and the rendezvous bootstrap multiaddrs
 * from disk (the same files the cloud and rendezvous use), filters the
 * bootstrap list to browser-dialable transports (`/ws`, `/wss`,
 * `/webtransport`), and exposes both as `VITE_*` env vars.
 *
 * The player has no manual configuration UI — if these are missing the
 * `/player` page renders an inline error pointing at `pnpm
 * app:rendezvous`.
 */

export interface PlayerIpfsConfig {
	bootstrapMultiaddrs: string[];
	swarmKey: string;
}

function parseBootstrap(raw: string | undefined): string[] {
	if (!raw) return [];
	const lines = raw
		.split(/[\n,]/)
		.map((s) => s.trim())
		.filter((s) => s.length > 0);
	return lines.filter(
		(addr) =>
			addr.includes('/ws/') ||
			addr.endsWith('/ws') ||
			addr.includes('/wss/') ||
			addr.endsWith('/wss') ||
			addr.includes('/webtransport')
	);
}

const bootstrapMultiaddrs = parseBootstrap(import.meta.env.VITE_RENDEZVOUS_BOOTSTRAP);
const swarmKey = (import.meta.env.VITE_SWARM_KEY ?? '').trim();
const swarmKeyValid = swarmKey.startsWith('/key/swarm/psk/1.0.0/');

export const playerIpfsConfig: PlayerIpfsConfig = {
	bootstrapMultiaddrs,
	swarmKey: swarmKeyValid ? swarmKey + (swarmKey.endsWith('\n') ? '' : '\n') : ''
};

export const playerIpfsConfigured = bootstrapMultiaddrs.length > 0 && swarmKeyValid;

export interface ConfigDiagnostic {
	bootstrapMultiaddrs: number;
	swarmKey: 'present' | 'missing' | 'invalid';
}

export const playerIpfsDiagnostic: ConfigDiagnostic = {
	bootstrapMultiaddrs: bootstrapMultiaddrs.length,
	swarmKey: !swarmKey ? 'missing' : swarmKeyValid ? 'present' : 'invalid'
};
