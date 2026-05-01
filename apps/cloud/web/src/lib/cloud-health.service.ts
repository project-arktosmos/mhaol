import { writable, type Writable } from 'svelte/store';

export interface CloudDbStatus {
	engine: string;
	namespace: string;
	database: string;
	connected: boolean;
	version: string | null;
}

export type PackageHealthStatus = 'ok' | 'warning' | 'error' | 'unavailable';

export interface PackageHealth {
	name: string;
	status: PackageHealthStatus;
	available: boolean;
	message?: string;
	details: Record<string, unknown>;
}

export interface PackagesHealth {
	p2pStream: PackageHealth;
	ytDlp: PackageHealth;
	torrent: PackageHealth;
	ed2k: PackageHealth;
	ipfs: PackageHealth;
}

export interface CloudStatus {
	status: string;
	version: string;
	started_at: number;
	now: number;
	uptime_seconds: number;
	host: string;
	port: number;
	local_ip: string | null;
	public_ip: string | null;
	signaling_address: string | null;
	client_address: string | null;
	db: CloudDbStatus;
	packages: PackagesHealth;
}

export interface CloudHealthState {
	loading: boolean;
	online: boolean;
	lastCheckedAt: number | null;
	latencyMs: number | null;
	status: CloudStatus | null;
	error: string | null;
}

const initialState: CloudHealthState = {
	loading: false,
	online: false,
	lastCheckedAt: null,
	latencyMs: null,
	status: null,
	error: null
};

class CloudHealthService {
	state: Writable<CloudHealthState> = writable(initialState);
	private timer: ReturnType<typeof setInterval> | null = null;

	async refresh(): Promise<void> {
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		const startedAt = performance.now();
		try {
			const res = await fetch('/api/cloud/status', { cache: 'no-store' });
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const status = (await res.json()) as CloudStatus;
			const latencyMs = Math.round(performance.now() - startedAt);
			this.state.set({
				loading: false,
				online: true,
				lastCheckedAt: Date.now(),
				latencyMs,
				status,
				error: null
			});
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Unknown error';
			this.state.update((s) => ({
				...s,
				loading: false,
				online: false,
				lastCheckedAt: Date.now(),
				latencyMs: null,
				error: message
			}));
		}
	}

	start(intervalMs: number = 5000): void {
		this.refresh();
		this.stop();
		this.timer = setInterval(() => this.refresh(), intervalMs);
	}

	stop(): void {
		if (this.timer !== null) {
			clearInterval(this.timer);
			this.timer = null;
		}
	}
}

export const cloudHealthService = new CloudHealthService();
