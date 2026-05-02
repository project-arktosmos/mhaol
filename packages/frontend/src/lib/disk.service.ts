import { writable, type Writable } from 'svelte/store';

export interface DiskInfo {
	name: string;
	mountPoint: string;
	fileSystem: string;
	kind: string;
	isRemovable: boolean;
	totalBytes: number;
	availableBytes: number;
	usedBytes: number;
	isDataRootDisk: boolean;
}

export interface SubdirInfo {
	name: string;
	path: string;
	kind: 'Dir' | 'File';
	exists: boolean;
	sizeBytes: number;
}

export interface DiskResponse {
	dataRoot: string;
	dataRootTotalBytes: number;
	dataRootMountPoint: string | null;
	disks: DiskInfo[];
	subdirs: SubdirInfo[];
}

export interface DiskState {
	loading: boolean;
	data: DiskResponse | null;
	error: string | null;
}

const initialState: DiskState = {
	loading: false,
	data: null,
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

class DiskService {
	state: Writable<DiskState> = writable(initialState);

	async refresh(): Promise<void> {
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const res = await fetch('/api/disk', { cache: 'no-store' });
			if (!res.ok) throw new Error(await parseError(res));
			const data = (await res.json()) as DiskResponse;
			this.state.set({ loading: false, data, error: null });
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Unknown error';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		}
	}
}

export const diskService = new DiskService();
