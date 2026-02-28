import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { ensureIdentity, regenerate } from '$lib/server/identities';

const IDENTITY_NAME = 'SIGNALING_WALLET';

export const GET: RequestHandler = async () => {
	const address = ensureIdentity(IDENTITY_NAME);
	return json({ name: IDENTITY_NAME, address });
};

export const DELETE: RequestHandler = async () => {
	const address = regenerate(IDENTITY_NAME);
	return json({ name: IDENTITY_NAME, address });
};
