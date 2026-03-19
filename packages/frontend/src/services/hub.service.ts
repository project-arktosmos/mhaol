import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl } from 'frontend/lib/api-base';
import type { HubApp } from 'frontend/types/hub.type';

interface HubState {
	loading: boolean;
	apps: HubApp[];
	error: string | null;
}

const initialState: HubState = {
	loading: false,
	apps: [],
	error: null
};

class HubService {
	public state: Writable<HubState> = writable(initialState);

	private _initialized = false;
	private _pollInterval: ReturnType<typeof setInterval> | null = null;

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;
		this._initialized = true;
		await this.refresh();
		this._pollInterval = setInterval(() => this.poll(), 10000);
	}

	destroy(): void {
		if (this._pollInterval) {
			clearInterval(this._pollInterval);
			this._pollInterval = null;
		}
	}

	async refresh(): Promise<void> {
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const res = await fetch(apiUrl('/api/hub'));
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const apps: HubApp[] = (await res.json()).map((a: HubApp) => ({
				...a,
				logs: a.logs ?? []
			}));
			this.state.update((s) => ({ ...s, loading: false, apps }));
			await this.poll();
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Failed to load apps';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		}
	}

	private async poll(): Promise<void> {
		let apps: HubApp[] = [];
		this.state.subscribe((s) => (apps = s.apps))();

		const updated = await Promise.all(
			apps.map(async (app) => {
				const [healthData, logs] = await Promise.all([
					this.fetchHealth(app.name),
					this.fetchLogs(app.name)
				]);
				return { ...app, status: healthData, logs };
			})
		);
		this.state.update((s) => ({ ...s, apps: updated }));
	}

	private async fetchHealth(name: string): Promise<HubApp['status']> {
		try {
			const res = await fetch(apiUrl(`/api/hub/${name}/health`));
			if (!res.ok) return 'unknown';
			const data = await res.json();
			return data.status as HubApp['status'];
		} catch {
			return 'unknown';
		}
	}

	private async fetchLogs(name: string): Promise<string[]> {
		try {
			const res = await fetch(apiUrl(`/api/hub/${name}/logs`));
			if (!res.ok) return [];
			const data = await res.json();
			return data.logs ?? [];
		} catch {
			return [];
		}
	}

	async startApp(name: string): Promise<void> {
		this.state.update((s) => ({
			...s,
			apps: s.apps.map((a) =>
				a.name === name ? { ...a, status: 'starting' as const, logs: [] } : a
			)
		}));

		try {
			const res = await fetch(apiUrl(`/api/hub/${name}/start`), { method: 'POST' });
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const data = await res.json();
			if (!data.success) {
				this.state.update((s) => ({ ...s, error: data.message }));
			}
		} catch (err) {
			const message = err instanceof Error ? err.message : `Failed to start ${name}`;
			this.state.update((s) => ({ ...s, error: message }));
		}
	}

	async stopApp(name: string): Promise<void> {
		try {
			const res = await fetch(apiUrl(`/api/hub/${name}/stop`), { method: 'POST' });
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			await this.refresh();
		} catch (err) {
			const message = err instanceof Error ? err.message : `Failed to stop ${name}`;
			this.state.update((s) => ({ ...s, error: message }));
		}
	}

	async dismissApp(name: string): Promise<void> {
		// Remove a failed process entry so the user can retry
		try {
			await fetch(apiUrl(`/api/hub/${name}/stop`), { method: 'POST' });
		} catch {
			// ignore
		}
		await this.refresh();
	}
}

export const hubService = new HubService();
