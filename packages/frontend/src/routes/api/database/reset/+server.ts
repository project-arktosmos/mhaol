import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { join } from 'node:path';
import { getDatabase, initializeSchema } from 'database';

export const POST: RequestHandler = async ({ locals }) => {
	const db = getDatabase();

	const triggers = db
		.prepare("SELECT name FROM sqlite_master WHERE type = 'trigger'")
		.all() as { name: string }[];

	for (const trigger of triggers) {
		db.exec(`DROP TRIGGER IF EXISTS "${trigger.name}"`);
	}

	const tables = db
		.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'")
		.all() as { name: string }[];

	for (const table of tables) {
		db.exec(`DROP TABLE IF EXISTS "${table.name}"`);
	}

	initializeSchema(db);

	// Re-seed default library
	const defaultDownloadsPath = join(process.env.HOME ?? '/tmp', 'Downloads');
	const libraryId = crypto.randomUUID();
	locals.libraryRepo.insert({
		id: libraryId,
		name: 'Downloads',
		path: defaultDownloadsPath,
		media_types: JSON.stringify(['video', 'images', 'music']),
		date_added: Date.now()
	});

	// Seed library references for both YouTube and torrent
	locals.metadataRepo.set('youtube.libraryId', libraryId);
	locals.metadataRepo.set('torrent.libraryId', libraryId);

	console.log('[database] Reset complete — all tables dropped, recreated, and reseeded');

	const newTables = db
		.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'")
		.all() as { name: string }[];

	return json({ ok: true, tables: newTables.map((t) => t.name) });
};
