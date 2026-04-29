import { writable, type Writable } from 'svelte/store';

export interface TableInfo {
	name: string;
	record_count: number;
}

export interface TablesResponse {
	namespace: string;
	database: string;
	tables: TableInfo[];
}

export interface RecordsResponse {
	table: string;
	limit: number;
	offset: number;
	total: number;
	records: unknown[];
}

export interface DatabaseState {
	loading: boolean;
	namespace: string;
	database: string;
	tables: TableInfo[];
	error: string | null;
}

export interface RecordsState {
	loading: boolean;
	table: string | null;
	records: unknown[];
	limit: number;
	offset: number;
	total: number;
	error: string | null;
}

const initialDbState: DatabaseState = {
	loading: false,
	namespace: '',
	database: '',
	tables: [],
	error: null
};

const initialRecordsState: RecordsState = {
	loading: false,
	table: null,
	records: [],
	limit: 100,
	offset: 0,
	total: 0,
	error: null
};

async function parseError(res: Response): Promise<string> {
	try {
		const data = await res.json();
		if (data && typeof data.error === 'string') return data.error;
	} catch {
		// fall through
	}
	return `HTTP ${res.status}`;
}

class DatabaseService {
	state: Writable<DatabaseState> = writable(initialDbState);
	records: Writable<RecordsState> = writable(initialRecordsState);

	async refresh(): Promise<void> {
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const res = await fetch('/api/database/tables', { cache: 'no-store' });
			if (!res.ok) throw new Error(await parseError(res));
			const data = (await res.json()) as TablesResponse;
			this.state.set({
				loading: false,
				namespace: data.namespace,
				database: data.database,
				tables: data.tables,
				error: null
			});
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Unknown error';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		}
	}

	async loadTable(table: string, limit = 100, offset = 0): Promise<void> {
		this.records.update((s) => ({ ...s, loading: true, error: null, table }));
		try {
			const url = `/api/database/tables/${encodeURIComponent(table)}?limit=${limit}&offset=${offset}`;
			const res = await fetch(url, { cache: 'no-store' });
			if (!res.ok) throw new Error(await parseError(res));
			const data = (await res.json()) as RecordsResponse;
			this.records.set({
				loading: false,
				table: data.table,
				records: data.records,
				limit: data.limit,
				offset: data.offset,
				total: data.total,
				error: null
			});
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Unknown error';
			this.records.update((s) => ({ ...s, loading: false, error: message }));
		}
	}

	clearTable(): void {
		this.records.set(initialRecordsState);
	}
}

export const databaseService = new DatabaseService();
