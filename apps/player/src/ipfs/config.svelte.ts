/**
 * Persisted IPFS configuration that the user provides at runtime: the
 * rendezvous bootstrap multiaddrs and the swarm-key contents.
 *
 * Stored in `localStorage` so the user only has to paste it once. The
 * player runs as a client-only SPA (`ssr: false`, `prerender: false`),
 * so the initial localStorage read can happen eagerly at module load —
 * lazy-loading inside the property getters would mutate `$state` from
 * within `$derived(...)` reads on the page and trip
 * `state_unsafe_mutation`.
 *
 * The swarm key default is empty — the user MUST paste theirs. There is
 * no sane default; it has to match the cloud / rendezvous PSK.
 */

const STORAGE_KEY = 'mhaol-player:ipfs-config';

interface StoredConfig {
	bootstrapMultiaddrs: string[];
	swarmKey: string;
}

const DEFAULTS: StoredConfig = {
	bootstrapMultiaddrs: [],
	swarmKey: ''
};

function load(): StoredConfig {
	if (typeof localStorage === 'undefined') return { ...DEFAULTS };
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (!raw) return { ...DEFAULTS };
		const parsed = JSON.parse(raw);
		return {
			bootstrapMultiaddrs: Array.isArray(parsed.bootstrapMultiaddrs)
				? parsed.bootstrapMultiaddrs.filter((s: unknown) => typeof s === 'string')
				: [],
			swarmKey: typeof parsed.swarmKey === 'string' ? parsed.swarmKey : ''
		};
	} catch {
		return { ...DEFAULTS };
	}
}

function save(config: StoredConfig): void {
	if (typeof localStorage === 'undefined') return;
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
	} catch {
		// quota exhausted / private window — silently drop
	}
}

class IpfsConfigStore {
	#bootstrapMultiaddrs: string[] = $state([]);
	#swarmKey: string = $state('');

	constructor(initial: StoredConfig) {
		this.#bootstrapMultiaddrs = initial.bootstrapMultiaddrs;
		this.#swarmKey = initial.swarmKey;
	}

	get bootstrapMultiaddrs(): string[] {
		return this.#bootstrapMultiaddrs;
	}

	get swarmKey(): string {
		return this.#swarmKey;
	}

	get configured(): boolean {
		return (
			this.#bootstrapMultiaddrs.length > 0 && this.#swarmKey.startsWith('/key/swarm/psk/1.0.0/')
		);
	}

	update(input: Partial<StoredConfig>): void {
		if (input.bootstrapMultiaddrs) {
			this.#bootstrapMultiaddrs = input.bootstrapMultiaddrs.filter((s) => s.trim().length > 0);
		}
		if (typeof input.swarmKey === 'string') {
			this.#swarmKey = input.swarmKey;
		}
		save({
			bootstrapMultiaddrs: this.#bootstrapMultiaddrs,
			swarmKey: this.#swarmKey
		});
	}

	clear(): void {
		this.#bootstrapMultiaddrs = [];
		this.#swarmKey = '';
		save({ bootstrapMultiaddrs: [], swarmKey: '' });
	}
}

export const ipfsConfigStore = new IpfsConfigStore(load());
