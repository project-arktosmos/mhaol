import { error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ url, locals }) => {
	const path = url.searchParams.get('path');
	if (!path) {
		throw error(400, 'Missing path parameter');
	}

	const res = await fetch(
		`${locals.imageTaggerBaseUrl}/images/serve?path=${encodeURIComponent(path)}`
	).catch(() => null);

	if (!res?.ok) {
		const status = res?.status ?? 503;
		const text = res ? await res.text() : 'Image tagger server not available';
		return new Response(text, { status });
	}

	return new Response(res.body, {
		headers: {
			'Content-Type': res.headers.get('Content-Type') ?? 'application/octet-stream',
			'Cache-Control': res.headers.get('Cache-Control') ?? 'public, max-age=3600'
		}
	});
};
