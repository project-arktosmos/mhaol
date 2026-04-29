import { error } from '@sveltejs/kit';
import type { Document } from '$lib/documents.service';

export const prerender = false;

export const load = async ({ params, fetch }) => {
	const res = await fetch(`/api/documents/${encodeURIComponent(params.ipfsHash)}`, {
		cache: 'no-store'
	});
	if (res.status === 404) {
		throw error(404, 'document not found');
	}
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
	const document = (await res.json()) as Document;
	return { document };
};
