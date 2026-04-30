import { error } from '@sveltejs/kit';

export const prerender = false;

interface CatalogGenre {
	id: string;
	name: string;
}

export const load = async ({ params, fetch }) => {
	const id = params.id;
	const res = await fetch('/api/catalog/retroachievements/genres', { cache: 'no-store' });
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
	const consoles = (await res.json()) as CatalogGenre[];
	const console = consoles.find((c) => c.id === id);
	if (!console) throw error(404, `console "${id}" not found`);
	return { console };
};
