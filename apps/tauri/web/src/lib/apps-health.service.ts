import { writable, type Writable } from 'svelte/store';

export interface AppEndpoint {
	id: 'cloud' | 'player';
	label: string;
	url: string;
	healthPath: string;
	expectsJson: boolean;
}

export const APP_ENDPOINTS: AppEndpoint[] = [
	{
		id: 'cloud',
		label: 'Cloud',
		url: 'http://localhost:9898',
		healthPath: '/api/cloud/status',
		expectsJson: true
	},
	{
		id: 'player',
		label: 'Player',
		url: 'http://localhost:9595',
		healthPath: '/',
		expectsJson: false
	}
];

export interface AppHealth {
	id: AppEndpoint['id'];
	label: string;
	url: string;
	online: boolean;
	loading: boolean;
	latencyMs: number | null;
	lastCheckedAt: number | null;
	error: string | null;
	info: Record<string, unknown> | null;
}

export interface AppsHealthState {
	apps: AppHealth[];
}

const initialApp = (e: AppEndpoint): AppHealth => ({
	id: e.id,
	label: e.label,
	url: e.url,
	online: false,
	loading: false,
	latencyMs: null,
	lastCheckedAt: null,
	error: null,
	info: null
});

class AppsHealthService {
	state: Writable<AppsHealthState> = writable({
		apps: APP_ENDPOINTS.map(initialApp)
	});
	private timer: ReturnType<typeof setInterval> | null = null;

	private async probe(endpoint: AppEndpoint): Promise<AppHealth> {
		const startedAt = performance.now();
		const target = `${endpoint.url}${endpoint.healthPath}`;
		try {
			const init: RequestInit = endpoint.expectsJson
				? { cache: 'no-store' }
				: { cache: 'no-store', mode: 'no-cors' };
			const res = await fetch(target, init);
			const latencyMs = Math.round(performance.now() - startedAt);

			if (endpoint.expectsJson) {
				if (!res.ok) throw new Error(`HTTP ${res.status}`);
				const info = (await res.json()) as Record<string, unknown>;
				return {
					id: endpoint.id,
					label: endpoint.label,
					url: endpoint.url,
					online: true,
					loading: false,
					latencyMs,
					lastCheckedAt: Date.now(),
					error: null,
					info
				};
			}

			return {
				id: endpoint.id,
				label: endpoint.label,
				url: endpoint.url,
				online: true,
				loading: false,
				latencyMs,
				lastCheckedAt: Date.now(),
				error: null,
				info: null
			};
		} catch (err) {
			return {
				id: endpoint.id,
				label: endpoint.label,
				url: endpoint.url,
				online: false,
				loading: false,
				latencyMs: null,
				lastCheckedAt: Date.now(),
				error: err instanceof Error ? err.message : 'Unknown error',
				info: null
			};
		}
	}

	async refresh(): Promise<void> {
		this.state.update((s) => ({
			apps: s.apps.map((a) => ({ ...a, loading: true }))
		}));
		const next = await Promise.all(APP_ENDPOINTS.map((e) => this.probe(e)));
		this.state.set({ apps: next });
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

export const appsHealthService = new AppsHealthService();
