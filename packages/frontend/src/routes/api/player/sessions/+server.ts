import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const STREAM_SERVER_URL = process.env.P2P_STREAM_URL ?? 'http://localhost:3001';

export const POST: RequestHandler = async ({ request }) => {
	try {
		const body = await request.json();
		const res = await fetch(`${STREAM_SERVER_URL}/sessions`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(body),
			signal: AbortSignal.timeout(5000)
		});

		if (!res.ok) {
			const err = await res.json().catch(() => ({}));
			return json(
				{ error: (err as { error?: string }).error ?? `Streaming server error: ${res.status}` },
				{ status: res.status }
			);
		}

		const data = await res.json();
		return json(data);
	} catch {
		return json({ error: 'Streaming server is not available' }, { status: 503 });
	}
};
