import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import type { SignalingStatusResponse } from '$types/signaling.type';

export const GET: RequestHandler = async ({ url }) => {
	const target = url.searchParams.get('url');
	if (!target) {
		return json({ error: 'Missing url query parameter' }, { status: 400 });
	}

	try {
		const res = await fetch(`${target}/status`, {
			signal: AbortSignal.timeout(3000)
		});

		if (!res.ok) {
			return json({ error: `Server returned ${res.status}` }, { status: 502 });
		}

		const data = (await res.json()) as SignalingStatusResponse;
		return json(data);
	} catch (err) {
		const message = err instanceof Error ? err.message : 'Connection failed';
		return json({ error: message }, { status: 502 });
	}
};
