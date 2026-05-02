/**
 * Runtime IPFS configuration fetched from the cloud's `/api/p2p/bootstrap`
 * endpoint. The cloud serves both the player and the bootstrap data, so the
 * fetch is same-origin in production. In dev, Vite proxies `/api` to the
 * cloud (see `apps/player/vite.config.ts`).
 *
 * The player has no manual configuration UI — if the fetch fails or the
 * payload is invalid, the `/player` page renders an inline error.
 */

export interface PlayerIpfsConfig {
	bootstrapMultiaddrs: string[];
	swarmKey: string;
}

export interface PlayerIpfsConfigResult {
	config: PlayerIpfsConfig | null;
	diagnostic: ConfigDiagnostic;
	error: string | null;
}

export interface ConfigDiagnostic {
	bootstrapMultiaddrs: number;
	swarmKey: 'present' | 'missing' | 'invalid';
}

interface BootstrapResponse {
	peerId?: string;
	swarmKey?: string;
	multiaddrs?: string[];
}

const BROWSER_TRANSPORT_RE = /\/(wss?|webtransport)(\/|$)/;

function filterBrowserDialable(addrs: string[] | undefined): string[] {
	if (!addrs) return [];
	return addrs
		.map((s) => s.trim())
		.filter((s) => s.length > 0 && BROWSER_TRANSPORT_RE.test(s));
}

export async function fetchPlayerIpfsConfig(): Promise<PlayerIpfsConfigResult> {
	let res: Response;
	try {
		res = await fetch('/api/p2p/bootstrap', { cache: 'no-store' });
	} catch (err) {
		return {
			config: null,
			diagnostic: { bootstrapMultiaddrs: 0, swarmKey: 'missing' },
			error: err instanceof Error ? err.message : String(err)
		};
	}

	if (!res.ok) {
		return {
			config: null,
			diagnostic: { bootstrapMultiaddrs: 0, swarmKey: 'missing' },
			error: `cloud /api/p2p/bootstrap returned ${res.status}`
		};
	}

	let body: BootstrapResponse;
	try {
		body = (await res.json()) as BootstrapResponse;
	} catch (err) {
		return {
			config: null,
			diagnostic: { bootstrapMultiaddrs: 0, swarmKey: 'missing' },
			error: err instanceof Error ? err.message : 'invalid bootstrap JSON'
		};
	}

	const bootstrapMultiaddrs = filterBrowserDialable(body.multiaddrs);
	const swarmKeyRaw = (body.swarmKey ?? '').trim();
	const swarmKeyValid = swarmKeyRaw.startsWith('/key/swarm/psk/1.0.0/');
	const swarmKey = swarmKeyValid ? swarmKeyRaw + '\n' : '';

	const diagnostic: ConfigDiagnostic = {
		bootstrapMultiaddrs: bootstrapMultiaddrs.length,
		swarmKey: !swarmKeyRaw ? 'missing' : swarmKeyValid ? 'present' : 'invalid'
	};

	if (bootstrapMultiaddrs.length === 0 || !swarmKeyValid) {
		return { config: null, diagnostic, error: null };
	}

	return {
		config: { bootstrapMultiaddrs, swarmKey },
		diagnostic,
		error: null
	};
}
