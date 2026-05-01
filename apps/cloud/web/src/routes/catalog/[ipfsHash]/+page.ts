import { error, redirect } from '@sveltejs/kit';
import { base } from '$app/paths';
import type { Firkin } from '$lib/firkins.service';

export const prerender = false;

export const load = async ({ params, fetch }) => {
	const id = params.ipfsHash;
	const res = await fetch(`/api/firkins/${encodeURIComponent(id)}`, {
		cache: 'no-store'
	});

	if (res.status === 404) {
		// The firkin may have been rolled forward to a new CID by the
		// torrent-completion watcher (which deletes the old record and
		// creates a new one whose `version_hashes` lists this id).
		// Walk the firkin list and redirect if a successor exists.
		const listRes = await fetch('/api/firkins', { cache: 'no-store' });
		if (listRes.ok) {
			const list = (await listRes.json()) as Firkin[];
			const successor = list.find((d) => (d.version_hashes ?? []).includes(id));
			if (successor) {
				throw redirect(307, `${base}/catalog/${encodeURIComponent(successor.id)}`);
			}
		}
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
