import { json, error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { getAll, getPassport, regenerate, getAddress } from '$lib/server/identities';

export const GET: RequestHandler = async () => {
	const identities = getAll();
	const entries = await Promise.all(
		Object.entries(identities).map(async ([name, address]) => {
			const passport = await getPassport(name);
			return {
				name,
				address,
				passport: JSON.stringify({
					raw: passport!.raw,
					hash: passport!.hash,
					signature: passport!.signature
				})
			};
		})
	);
	return json(entries);
};

export const POST: RequestHandler = async ({ request }) => {
	const body = (await request.json()) as { name?: string };

	if (!body.name || typeof body.name !== 'string') {
		return error(400, 'Missing or invalid "name" field');
	}

	const name = body.name.trim().toUpperCase().replace(/[^A-Z0-9_]/g, '_');
	if (!name) {
		return error(400, 'Name must contain at least one alphanumeric character');
	}

	if (getAddress(name)) {
		return error(409, `Identity "${name}" already exists`);
	}

	const address = regenerate(name);
	return json({ name, address }, { status: 201 });
};
