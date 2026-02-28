import type { RequestHandler } from './$types';
import { proxyToImageTagger } from '$lib/server/image-tagger-proxy';

export const POST: RequestHandler = async ({ request, locals }) => {
	const body = await request.text();
	return proxyToImageTagger(locals.imageTaggerBaseUrl, '/images/tag', {
		method: 'POST',
		body
	});
};
