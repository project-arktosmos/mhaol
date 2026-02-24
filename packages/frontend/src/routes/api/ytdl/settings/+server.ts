import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const YOUTUBE_SETTINGS_KEYS = [
	'youtube.downloadMode',
	'youtube.defaultQuality',
	'youtube.defaultFormat',
	'youtube.defaultVideoQuality',
	'youtube.defaultVideoFormat',
	'youtube.outputPath',
	'youtube.poToken',
	'youtube.cookies'
] as const;

function getDefaults(outputDir: string): Record<string, string> {
	return {
		'youtube.downloadMode': 'audio',
		'youtube.defaultQuality': 'high',
		'youtube.defaultFormat': 'aac',
		'youtube.defaultVideoQuality': 'best',
		'youtube.defaultVideoFormat': 'mp4',
		'youtube.outputPath': outputDir,
		'youtube.poToken': '',
		'youtube.cookies': ''
	};
}

export const GET: RequestHandler = async ({ locals }) => {
	const rows = locals.settingsRepo.getByPrefix('youtube.');
	const defaults = getDefaults(locals.ytdlOutputDir);

	// Seed missing keys with defaults
	const existing = new Map(rows.map((r) => [r.key, r.value]));
	const missing: Record<string, string> = {};
	for (const key of YOUTUBE_SETTINGS_KEYS) {
		if (!existing.has(key)) {
			missing[key] = defaults[key];
		}
	}

	if (Object.keys(missing).length > 0) {
		locals.settingsRepo.setMany(missing);
	}

	return json({
		downloadMode: existing.get('youtube.downloadMode') ?? defaults['youtube.downloadMode'],
		defaultQuality: existing.get('youtube.defaultQuality') ?? defaults['youtube.defaultQuality'],
		defaultFormat: existing.get('youtube.defaultFormat') ?? defaults['youtube.defaultFormat'],
		defaultVideoQuality:
			existing.get('youtube.defaultVideoQuality') ?? defaults['youtube.defaultVideoQuality'],
		defaultVideoFormat:
			existing.get('youtube.defaultVideoFormat') ?? defaults['youtube.defaultVideoFormat'],
		outputPath: existing.get('youtube.outputPath') ?? defaults['youtube.outputPath'],
		poToken: existing.get('youtube.poToken') ?? defaults['youtube.poToken'],
		cookies: existing.get('youtube.cookies') ?? defaults['youtube.cookies']
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
	if (body.outputPath !== undefined) entries['youtube.outputPath'] = body.outputPath;
	if (body.poToken !== undefined) entries['youtube.poToken'] = body.poToken;
	if (body.cookies !== undefined) entries['youtube.cookies'] = body.cookies;

	if (Object.keys(entries).length === 0) {
		return json({ error: 'No valid settings provided' }, { status: 400 });
	}

	locals.settingsRepo.setMany(entries);

	// Sync relevant config to the Rust server
	const rustConfig: Record<string, unknown> = {};
	if (body.outputPath !== undefined) rustConfig.outputPath = body.outputPath;
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

	return json({ ok: true });
};
