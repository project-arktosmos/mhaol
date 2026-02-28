import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const PUT: RequestHandler = async ({ request, locals }) => {
	const body = await request.json();
	const { plugin, key, value } = body as { plugin?: string; key?: string; value?: string };

	if (!plugin || !key || value === undefined) {
		return json({ error: 'Missing required fields: plugin, key, value' }, { status: 400 });
	}

	const updated = locals.pluginConnector.updatePluginSetting(plugin, key, value);
	if (!updated) {
		return json({ error: 'Invalid plugin name or setting key' }, { status: 404 });
	}

	return json({ ok: true });
};
