import { writable, type Writable } from 'svelte/store';

export interface IpfsPin {
	id: string;
	cid: string;
	path: string;
	mime: string;
	size: number;
	created_at: string;
}

export interface IpfsPinsState {
	loading: boolean;
	pins: IpfsPin[];
	error: string | null;
}

const initialState: IpfsPinsState = {
	loading: false,
	pins: [],
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

class IpfsService {
	state: Writable<IpfsPinsState> = writable(initialState);

	async refresh(): Promise<void> {
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const res = await fetch('/api/ipfs/pins', { cache: 'no-store' });
			if (!res.ok) throw new Error(await parseError(res));
			const pins = (await res.json()) as IpfsPin[];
			this.state.set({ loading: false, pins, error: null });
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Unknown error';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		}
	}
}

export const ipfsService = new IpfsService();
