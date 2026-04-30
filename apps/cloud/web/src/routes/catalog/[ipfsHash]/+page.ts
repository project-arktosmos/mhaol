import { error } from '@sveltejs/kit';
import type { Firkin } from '$lib/firkins.service';

export const prerender = false;

export const load = async ({ params, fetch }) => {
	const res = await fetch(`/api/firkins/${encodeURIComponent(params.ipfsHash)}`, {
		cache: 'no-store'
	});
	if (res.status === 404) {
		throw error(404, 'firkin not found');
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
	const firkin = (await res.json()) as Firkin;
	return { firkin };
};
