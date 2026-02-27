import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const PIRATEBAY_API = 'https://apibay.org';

export const GET: RequestHandler = async ({ url }) => {
	const query = url.searchParams.get('q');
	const category = url.searchParams.get('cat') ?? '0';

	if (!query || !query.trim()) {
		return json({ error: 'Missing q parameter' }, { status: 400 });
	}

	try {
		const apiUrl = `${PIRATEBAY_API}/q.php?q=${encodeURIComponent(query.trim())}&cat=${encodeURIComponent(category)}`;
		const response = await fetch(apiUrl, {
			headers: { 'User-Agent': 'Mozilla/5.0 (compatible; Mhaol/1.0)' },
			signal: AbortSignal.timeout(30000)
		});

		if (!response.ok) {
			return json({ error: `PirateBay API returned ${response.status}` }, { status: 502 });
		}

		const data = await response.json();
		return json(data);
	} catch (err) {
		const message = err instanceof Error ? err.message : String(err);
		return json({ error: message }, { status: 500 });
	}
};
