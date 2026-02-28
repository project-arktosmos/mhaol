import { json } from '@sveltejs/kit';

export async function proxyToTorrent(
	baseUrl: string,
	path: string,
	init?: RequestInit
): Promise<Response> {
	try {
		const res = await fetch(`${baseUrl}${path}`, {
			...init,
			headers: {
				'Content-Type': 'application/json',
				...init?.headers
			}
		});

		if (!res.ok) {
			const body = await res.json().catch(() => ({ error: `HTTP ${res.status}` }));
			return json(body, { status: res.status });
		}

		const data = await res.json();
		return json(data, { status: res.status });
	} catch {
		return json({ error: 'Torrent server is not available' }, { status: 503 });
	}
}
