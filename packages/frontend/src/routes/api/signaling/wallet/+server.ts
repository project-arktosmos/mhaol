import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { generatePrivateKey, privateKeyToAccount } from 'viem/accounts';

const WALLET_KEY = 'signaling.wallet.privateKey';

export const GET: RequestHandler = async ({ locals }) => {
	let privateKey = locals.settingsRepo.get(WALLET_KEY);

	if (!privateKey) {
		privateKey = generatePrivateKey();
		locals.settingsRepo.set(WALLET_KEY, privateKey);
	}

	const account = privateKeyToAccount(privateKey as `0x${string}`);

	return json({ privateKey, address: account.address });
};
