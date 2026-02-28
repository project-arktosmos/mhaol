import type { Handle } from '@sveltejs/kit';

// Static/mobile builds (adapter-static) don't serve API routes,
// so skip the entire server initialization.
const isStaticBuild =
	process.env.TAURI_ENV_PLATFORM === 'android' || process.env.BUILD_ADAPTER === 'static';

// CORS: allowed origins for remote mobile clients
const CORS_ORIGINS = [
	'https://tauri.localhost',
	'tauri://localhost',
	...(process.env.ALLOWED_ORIGINS ?? '').split(',').filter(Boolean)
];

let initialized = false;
let connector: import('$lib/server/plugins/connector').PluginConnector;
let settingsRepo: import('database/repositories').SettingsRepository;
let metadataRepo: import('database/repositories').MetadataRepository;
let libraryRepo: import('database/repositories').LibraryRepository;
let libraryItemRepo: import('database/repositories').LibraryItemRepository;
let libraryItemLinkRepo: import('database/repositories').LibraryItemLinkRepository;
let mediaTypeRepo: import('database/repositories').MediaTypeRepository;
let categoryRepo: import('database/repositories').CategoryRepository;
let linkSourceRepo: import('database/repositories').LinkSourceRepository;

async function initializeServer() {
	if (initialized || isStaticBuild) return;
	initialized = true;

	const { join } = await import('node:path');
	const { getDatabase } = await import('database');
	const repos = await import('database/repositories');
	const { PluginConnector } = await import('$lib/server/plugins/connector');

	const dbPath = process.env.DB_PATH ?? undefined;
	const db = getDatabase(dbPath ? { dbPath } : undefined);
	settingsRepo = new repos.SettingsRepository(db);
	metadataRepo = new repos.MetadataRepository(db);
	libraryRepo = new repos.LibraryRepository(db);
	libraryItemRepo = new repos.LibraryItemRepository(db);
	libraryItemLinkRepo = new repos.LibraryItemLinkRepository(db);
	mediaTypeRepo = new repos.MediaTypeRepository(db);
	categoryRepo = new repos.CategoryRepository(db);

	// Seed a default Downloads library if none exist
	if (libraryRepo.getAll().length === 0) {
		const defaultDownloadsPath = join(process.env.HOME ?? '/tmp', 'Downloads');
		libraryRepo.insert({
			id: crypto.randomUUID(),
			name: 'Downloads',
			path: defaultDownloadsPath,
			media_types: JSON.stringify(['video', 'image', 'audio']),
			date_added: Date.now()
		});
		console.log(`[database] Created default library at ${defaultDownloadsPath}`);
	}

	console.log(`[database] Initialized`);

	// Plugin connector — handles all plugin lifecycle
	connector = new PluginConnector(db, { settingsRepo, metadataRepo, libraryRepo });

	type Manifest = import('$lib/server/plugins/types').PluginManifest;

	const ytDownloadManifest = (await import('$lib/server/plugins/definitions/yt-download.plugin.json')).default as unknown as Manifest;
	const { ytDownloadCompanion } = await import('$lib/server/plugins/definitions/yt-download.plugin');
	const p2pStreamManifest = (await import('$lib/server/plugins/definitions/p2p-stream.plugin.json')).default as unknown as Manifest;
	const torrentManifest = (await import('$lib/server/plugins/definitions/torrent.plugin.json')).default as unknown as Manifest;
	const { torrentCompanion } = await import('$lib/server/plugins/definitions/torrent.plugin');
	const imageTaggerManifest = (await import('$lib/server/plugins/definitions/image-tagger.plugin.json')).default as unknown as Manifest;
	const signalingManifest = (await import('$lib/server/plugins/definitions/signaling.plugin.json')).default as unknown as Manifest;
	const { signalingCompanion } = await import('$lib/server/plugins/definitions/signaling.plugin');
	const torrentSearchManifest = (await import('$lib/server/plugins/definitions/torrent-search.plugin.json')).default as unknown as Manifest;
	const { torrentSearchCompanion } = await import('$lib/server/plugins/definitions/torrent-search.plugin');
	const tmdbManifest = (await import('$lib/server/plugins/definitions/tmdb.plugin.json')).default as unknown as Manifest;
	const { tmdbCompanion } = await import('$lib/server/plugins/definitions/tmdb.plugin');

	connector.register(ytDownloadManifest, ytDownloadCompanion);
	connector.register(p2pStreamManifest);
	connector.register(torrentManifest, torrentCompanion);
	connector.register(imageTaggerManifest);
	connector.register(signalingManifest, signalingCompanion);
	connector.register(torrentSearchManifest, torrentSearchCompanion);
	connector.register(tmdbManifest, tmdbCompanion);
	await connector.initialize();

	// Cleanup on process exit
	process.on('exit', () => connector.shutdown());
	process.on('SIGINT', () => {
		connector.shutdown();
		process.exit(0);
	});
	process.on('SIGTERM', () => {
		connector.shutdown();
		process.exit(0);
	});
}

// Initialize immediately for server (adapter-node) builds
if (!isStaticBuild) {
	await initializeServer();
}

export const handle: Handle = async ({ event, resolve }) => {
	const origin = event.request.headers.get('origin');
	const isAllowedOrigin = origin != null && CORS_ORIGINS.includes(origin);

	// Handle CORS preflight
	if (event.request.method === 'OPTIONS' && isAllowedOrigin) {
		return new Response(null, {
			headers: {
				'Access-Control-Allow-Origin': origin,
				'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
				'Access-Control-Allow-Headers': 'Content-Type'
			}
		});
	}

	// Ensure server is initialized (no-op if already done)
	await initializeServer();

	if (initialized && connector) {
		// Plugin connector
		event.locals.pluginConnector = connector;

		// Plugin-provided locals (ytdl, p2p-stream, torrent, image-tagger)
		Object.assign(event.locals, connector.getLocals());

		// Core repos
		event.locals.settingsRepo = settingsRepo;
		event.locals.metadataRepo = metadataRepo;
		event.locals.libraryRepo = libraryRepo;
		event.locals.libraryItemRepo = libraryItemRepo;
		event.locals.libraryItemLinkRepo = libraryItemLinkRepo;
		event.locals.mediaTypeRepo = mediaTypeRepo;
		event.locals.categoryRepo = categoryRepo;
		event.locals.linkSourceRepo = connector.getLinkSourceRepo();
	}

	const response = await resolve(event);

	// Add CORS headers for allowed origins
	if (isAllowedOrigin) {
		response.headers.set('Access-Control-Allow-Origin', origin);
	}

	return response;
};
