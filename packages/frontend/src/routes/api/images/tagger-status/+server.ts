import type { RequestHandler } from './$types';
import { proxyToImageTagger } from '$lib/server/image-tagger-proxy';

export const GET: RequestHandler = async ({ locals }) => {
	return proxyToImageTagger(locals.imageTaggerBaseUrl, '/status');
};
