import { json } from '@sveltejs/kit';
import { getDefaultAddress } from '$lib/server/identities';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ locals }) => {
	const devUrl = locals.signalingDevUrl;
	const devAvailable = locals.signalingDevAvailable;
	const partyUrl = locals.signalingPartyUrl;

	const address = getDefaultAddress();
	const shortAddr = address ? address.toLowerCase().slice(2, 10) : null;
	const deployName = shortAddr ? `${shortAddr}-mhaol-signaling` : '';

	let deployedAvailable = false;
	if (partyUrl) {
		try {
			const controller = new AbortController();
			const timeout = setTimeout(() => controller.abort(), 3000);
			const res = await fetch(partyUrl, { signal: controller.signal });
			clearTimeout(timeout);
			deployedAvailable = res.ok || res.status === 404;
		} catch {
			// unreachable
		}
	}

	return json({ devAvailable, deployedAvailable, devUrl, partyUrl, deployName, identityAddress: address });
};
