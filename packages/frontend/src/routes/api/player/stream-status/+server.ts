import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const STREAM_SERVER_URL = process.env.P2P_STREAM_URL ?? 'http://localhost:3001';

export const GET: RequestHandler = async () => {
	try {
		const res = await fetch(`${STREAM_SERVER_URL}/health`, {
			signal: AbortSignal.timeout(2000)
		});
		await res.json();
		return json({ available: true });
	} catch {
		return json({ available: false });
	}
};
