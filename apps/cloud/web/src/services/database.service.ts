import { writable, type Writable } from 'svelte/store';
import { fetchRaw } from '$transport/fetch-helpers';
import type { DatabaseTable, DatabaseTableDetail } from '$types/database.type';

interface DatabaseState {
	tables: DatabaseTable[];
	selectedTable: DatabaseTableDetail | null;
	loading: boolean;
	error: string | null;
}

const initialState: DatabaseState = {
	tables: [],
	selectedTable: null,
	loading: false,
	error: null
};

class DatabaseService {
	public state: Writable<DatabaseState> = writable(initialState);

	async fetchTables(): Promise<void> {
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const res = await fetchRaw('/api/database/tables');
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const tables: DatabaseTable[] = await res.json();
			this.state.update((s) => ({ ...s, loading: false, tables }));
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Failed to load tables';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		}
	}

	async fetchTableRows(name: string, page = 1, limit = 20): Promise<void> {
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const res = await fetchRaw(`/api/database/tables/${name}?page=${page}&limit=${limit}`);
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const detail: DatabaseTableDetail = await res.json();
			this.state.update((s) => ({ ...s, loading: false, selectedTable: detail }));
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Failed to load table rows';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		}
	}

	clearSelection(): void {
		this.state.update((s) => ({ ...s, selectedTable: null }));
	}
}

export const databaseService = new DatabaseService();
