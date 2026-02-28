import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const DELETE: RequestHandler = async ({ params, locals }) => {
	const bridge = locals.p2pWorkerBridge;
	if (!bridge?.isAvailable()) {
		return json({ error: 'Streaming worker is not available' }, { status: 503 });
	}

	try {
		await bridge.deleteSession(params.id);
		return new Response(null, { status: 204 });
	} catch {
		return json({ error: 'Failed to delete session' }, { status: 500 });
	}
};
