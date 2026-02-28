import { json, error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { privateKeyToAccount } from 'viem/accounts';
import { getPrivateKey } from '$lib/server/identities';

const IDENTITY_NAME = 'SIGNALING_WALLET';

export const POST: RequestHandler = async ({ request }) => {
	const body = (await request.json()) as { message?: string };

	if (!body.message || typeof body.message !== 'string') {
		return error(400, 'Missing or invalid "message" field');
	}

	const privateKey = getPrivateKey(IDENTITY_NAME);
	if (!privateKey) {
		return error(404, 'Identity not found');
	}

	const account = privateKeyToAccount(privateKey as `0x${string}`);
	const signature = await account.signMessage({ message: body.message });

	return json({ signature });
};
