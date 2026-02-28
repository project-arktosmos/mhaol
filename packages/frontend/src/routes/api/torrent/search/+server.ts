import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import type { TorrentCategory } from 'torrent-search-thepiratebay/types';

export const GET: RequestHandler = async ({ url, locals }) => {
	const query = url.searchParams.get('q');
	const category = (url.searchParams.get('cat') ?? '0') as TorrentCategory;

	if (!query || !query.trim()) {
		return json({ error: 'Missing q parameter' }, { status: 400 });
	}

	try {
		const results = await locals.torrentSearch(query, { category });
		return json(results);
	} catch (err) {
		const message = err instanceof Error ? err.message : String(err);
		return json({ error: message }, { status: 500 });
	}
};
