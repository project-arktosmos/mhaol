import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ request, locals }) => {
	return locals.torrentBroadcaster.createStream(request);
};
