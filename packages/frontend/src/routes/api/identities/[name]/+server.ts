import { json, error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { getAddress, regenerate, remove } from '$lib/server/identities';

export const PUT: RequestHandler = async ({ params }) => {
	const { name } = params;

	if (!getAddress(name)) {
		return error(404, `Identity "${name}" not found`);
	}

	const address = regenerate(name);
	return json({ name, address });
};

export const DELETE: RequestHandler = async ({ params }) => {
	const { name } = params;

	if (!remove(name)) {
		return error(404, `Identity "${name}" not found`);
	}

	return json({ ok: true });
};
