import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const PUT: RequestHandler = async ({ request, locals }) => {
	const body = await request.json();
	const { addon, key, value } = body as { addon?: string; key?: string; value?: string };

	if (!addon || !key || value === undefined) {
		return json({ error: 'Missing required fields: addon, key, value' }, { status: 400 });
	}

	const updated = locals.pluginConnector.updatePluginSetting(addon, key, value);
	if (!updated) {
		return json({ error: 'Invalid addon name or setting key' }, { status: 404 });
	}

	return json({ ok: true });
};
