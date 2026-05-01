export interface DirEntry {
	name: string;
	path: string;
}

export interface BrowseResponse {
	path: string;
	parent: string | null;
	home: string;
	separator: string;
	roots: DirEntry[];
	entries: DirEntry[];
}

async function parseError(res: Response): Promise<string> {
	try {
		const data = await res.json();
		if (data && typeof data.error === 'string') return data.error;
	} catch {
		// fall through
	}
	return `HTTP ${res.status}`;
}

export async function browseDirectory(path?: string): Promise<BrowseResponse> {
	const url = path ? `/api/fs/browse?path=${encodeURIComponent(path)}` : '/api/fs/browse';
	const res = await fetch(url, { cache: 'no-store' });
	if (!res.ok) throw new Error(await parseError(res));
	return (await res.json()) as BrowseResponse;
}
