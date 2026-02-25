import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const YOUTUBE_SETTINGS_KEYS = [
	'youtube.downloadMode',
	'youtube.defaultQuality',
	'youtube.defaultFormat',
	'youtube.defaultVideoQuality',
	'youtube.defaultVideoFormat',
	'youtube.poToken',
	'youtube.cookies'
] as const;

const DEFAULTS: Record<string, string> = {
	'youtube.downloadMode': 'audio',
	'youtube.defaultQuality': 'high',
	'youtube.defaultFormat': 'aac',
	'youtube.defaultVideoQuality': 'best',
	'youtube.defaultVideoFormat': 'mp4',
	'youtube.poToken': '',
	'youtube.cookies': ''
};

export const GET: RequestHandler = async ({ locals }) => {
	const rows = locals.settingsRepo.getByPrefix('youtube.');

	// Seed missing keys with defaults
	const existing = new Map(rows.map((r) => [r.key, r.value]));
	const missing: Record<string, string> = {};
	for (const key of YOUTUBE_SETTINGS_KEYS) {
		if (!existing.has(key)) {
			missing[key] = DEFAULTS[key];
		}
	}

	if (Object.keys(missing).length > 0) {
		locals.settingsRepo.setMany(missing);
	}

	// Read libraryId from metadata
	const libraryId = (locals.metadataRepo.getValue<string>('youtube.libraryId') ?? '') as string;

	return json({
		downloadMode: existing.get('youtube.downloadMode') ?? DEFAULTS['youtube.downloadMode'],
		defaultQuality: existing.get('youtube.defaultQuality') ?? DEFAULTS['youtube.defaultQuality'],
		defaultFormat: existing.get('youtube.defaultFormat') ?? DEFAULTS['youtube.defaultFormat'],
		defaultVideoQuality:
			existing.get('youtube.defaultVideoQuality') ?? DEFAULTS['youtube.defaultVideoQuality'],
		defaultVideoFormat:
			existing.get('youtube.defaultVideoFormat') ?? DEFAULTS['youtube.defaultVideoFormat'],
		libraryId,
		poToken: existing.get('youtube.poToken') ?? DEFAULTS['youtube.poToken'],
		cookies: existing.get('youtube.cookies') ?? DEFAULTS['youtube.cookies']
	});
};

export const PUT: RequestHandler = async ({ request, locals }) => {
	const body = await request.json();
	const entries: Record<string, string> = {};

	if (body.downloadMode !== undefined) entries['youtube.downloadMode'] = body.downloadMode;
	if (body.defaultQuality !== undefined) entries['youtube.defaultQuality'] = body.defaultQuality;
	if (body.defaultFormat !== undefined) entries['youtube.defaultFormat'] = body.defaultFormat;
	if (body.defaultVideoQuality !== undefined)
		entries['youtube.defaultVideoQuality'] = body.defaultVideoQuality;
	if (body.defaultVideoFormat !== undefined)
		entries['youtube.defaultVideoFormat'] = body.defaultVideoFormat;
	if (body.poToken !== undefined) entries['youtube.poToken'] = body.poToken;
	if (body.cookies !== undefined) entries['youtube.cookies'] = body.cookies;

	if (Object.keys(entries).length > 0) {
		locals.settingsRepo.setMany(entries);
	}

	// Handle libraryId — stored in metadata, path synced to Rust
	const rustConfig: Record<string, unknown> = {};

	if (body.libraryId !== undefined) {
		locals.metadataRepo.set('youtube.libraryId', body.libraryId as string);
		const lib = locals.libraryRepo.get(body.libraryId as string);
		if (lib) {
			rustConfig.outputPath = lib.path;
		}
	}

	if (body.poToken !== undefined) rustConfig.poToken = body.poToken;
	if (body.cookies !== undefined) rustConfig.cookies = body.cookies;

	if (Object.keys(rustConfig).length > 0) {
		try {
			await fetch(`${locals.ytdlBaseUrl}/api/config`, {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(rustConfig)
			});
		} catch {
			console.warn('[ytdl-settings] Failed to sync config to Rust server');
		}
	}

	if (Object.keys(entries).length === 0 && body.libraryId === undefined) {
		return json({ error: 'No valid settings provided' }, { status: 400 });
	}

	return json({ ok: true });
};
