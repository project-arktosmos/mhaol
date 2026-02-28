import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ locals }) => {
	const all = locals.pluginConnector.getStatus();
	return json(all.filter((p) => p.source === 'addon'));
};
