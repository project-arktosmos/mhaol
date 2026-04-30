import { error } from '@sveltejs/kit';
import type { Artist } from '$lib/artists.service';

export const prerender = false;

export const load = async ({ params, fetch }) => {
	const id = params.ipfs;
	const res = await fetch(`/api/artists/${encodeURIComponent(id)}`, { cache: 'no-store' });
	if (res.status === 404) throw error(404, 'artist not found');
	if (!res.ok) {
		let message = `HTTP ${res.status}`;
		try {
			const body = await res.json();
			if (body && typeof body.error === 'string') message = body.error;
		} catch {
			// fall through
		}
		throw error(res.status, message);
	}
	const artist = (await res.json()) as Artist;
	return { artist };
};
