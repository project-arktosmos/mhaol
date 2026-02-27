import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const P2P_STREAM_SETTINGS_KEYS = [
	'p2p-stream.stunServer',
	'p2p-stream.turnServers',
	'p2p-stream.videoCodec',
	'p2p-stream.audioCodec',
	'p2p-stream.defaultStreamMode',
	'p2p-stream.videoQuality'
] as const;

const DEFAULTS: Record<string, string> = {
	'p2p-stream.stunServer': 'stun:stun.l.google.com:19302',
	'p2p-stream.turnServers': '[]',
	'p2p-stream.videoCodec': 'vp8',
	'p2p-stream.audioCodec': 'opus',
	'p2p-stream.defaultStreamMode': 'video',
	'p2p-stream.videoQuality': 'native'
};

export const GET: RequestHandler = async ({ locals }) => {
	const rows = locals.settingsRepo.getByPrefix('p2p-stream.');

	// Seed missing keys with defaults
	const existing = new Map(rows.map((r) => [r.key, r.value]));
	const missing: Record<string, string> = {};
	for (const key of P2P_STREAM_SETTINGS_KEYS) {
		if (!existing.has(key)) {
			missing[key] = DEFAULTS[key];
		}
	}

	if (Object.keys(missing).length > 0) {
		locals.settingsRepo.setMany(missing);
	}

	const turnServersRaw =
		existing.get('p2p-stream.turnServers') ?? DEFAULTS['p2p-stream.turnServers'];
	let turnServers: string[];
	try {
		turnServers = JSON.parse(turnServersRaw);
	} catch {
		turnServers = [];
	}

	return json({
		stunServer: existing.get('p2p-stream.stunServer') ?? DEFAULTS['p2p-stream.stunServer'],
		turnServers,
		videoCodec: existing.get('p2p-stream.videoCodec') ?? DEFAULTS['p2p-stream.videoCodec'],
		audioCodec: existing.get('p2p-stream.audioCodec') ?? DEFAULTS['p2p-stream.audioCodec'],
		defaultStreamMode:
			existing.get('p2p-stream.defaultStreamMode') ?? DEFAULTS['p2p-stream.defaultStreamMode'],
		videoQuality: existing.get('p2p-stream.videoQuality') ?? DEFAULTS['p2p-stream.videoQuality']
	});
};

export const PUT: RequestHandler = async ({ request, locals }) => {
	const body = await request.json();
	const entries: Record<string, string> = {};

	if (body.stunServer !== undefined) entries['p2p-stream.stunServer'] = body.stunServer;
	if (body.turnServers !== undefined)
		entries['p2p-stream.turnServers'] = JSON.stringify(body.turnServers);
	if (body.videoCodec !== undefined) entries['p2p-stream.videoCodec'] = body.videoCodec;
	if (body.audioCodec !== undefined) entries['p2p-stream.audioCodec'] = body.audioCodec;
	if (body.defaultStreamMode !== undefined)
		entries['p2p-stream.defaultStreamMode'] = body.defaultStreamMode;
	if (body.videoQuality !== undefined) entries['p2p-stream.videoQuality'] = body.videoQuality;

	if (Object.keys(entries).length === 0) {
		return json({ error: 'No valid settings provided' }, { status: 400 });
	}

	locals.settingsRepo.setMany(entries);
	return json({ ok: true });
};
