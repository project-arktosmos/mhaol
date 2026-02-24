import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const STREAM_SERVER_URL = process.env.P2P_STREAM_URL ?? 'http://localhost:3001';

export const DELETE: RequestHandler = async ({ params }) => {
	try {
		const res = await fetch(`${STREAM_SERVER_URL}/sessions/${params.id}`, {
			method: 'DELETE',
			signal: AbortSignal.timeout(5000)
		});

		return new Response(null, { status: res.status });
	} catch {
		return json({ error: 'Streaming server is not available' }, { status: 503 });
	}
};
