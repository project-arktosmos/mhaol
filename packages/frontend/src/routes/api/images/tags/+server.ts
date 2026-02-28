import type { RequestHandler } from './$types';
import { proxyToImageTagger } from '$lib/server/image-tagger-proxy';

export const POST: RequestHandler = async ({ request, locals }) => {
	const body = await request.text();
	return proxyToImageTagger(locals.imageTaggerBaseUrl, '/images/tags', {
		method: 'POST',
		body
	});
};

export const DELETE: RequestHandler = async ({ request, locals }) => {
	const body = await request.text();
	return proxyToImageTagger(locals.imageTaggerBaseUrl, '/images/tags', {
		method: 'DELETE',
		body
	});
};
