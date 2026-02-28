import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ locals }) => {
	const available = locals.p2pWorkerBridge?.isAvailable() ?? false;
	return json({ available });
};
