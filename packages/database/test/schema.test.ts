import { describe, it, expect, afterEach } from 'vitest';
import Database from 'better-sqlite3';
import { initializeSchema } from '../src/schema.js';

function createTestDb() {
	const db = new Database(':memory:');
	db.pragma('foreign_keys = ON');
	initializeSchema(db);
	return db;
}

describe('schema initialization', () => {
	let db: InstanceType<typeof Database>;

	afterEach(() => {
		db?.close();
	});

	it('should create the settings table', () => {
		db = createTestDb();
		const tables = db
			.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='settings'")
			.all();
		expect(tables).toHaveLength(1);
	});

	it('should create the metadata table', () => {
		db = createTestDb();
		const tables = db
			.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='metadata'")
			.all();
		expect(tables).toHaveLength(1);
	});

	it('should create the settings_updated_at trigger', () => {
		db = createTestDb();
		const triggers = db
			.prepare(
				"SELECT name FROM sqlite_master WHERE type='trigger' AND name='settings_updated_at'"
			)
			.all();
		expect(triggers).toHaveLength(1);
	});

	it('should create the metadata_updated_at trigger', () => {
		db = createTestDb();
		const triggers = db
			.prepare(
				"SELECT name FROM sqlite_master WHERE type='trigger' AND name='metadata_updated_at'"
			)
			.all();
		expect(triggers).toHaveLength(1);
	});

	it('should seed db_version metadata', () => {
		db = createTestDb();
		const row = db.prepare("SELECT * FROM metadata WHERE key = 'db_version'").get() as {
			key: string;
			value: string;
			type: string;
		};
		expect(row).toBeDefined();
		expect(row.value).toBe('19');
		expect(row.type).toBe('number');
	});

	it('should seed created_at metadata', () => {
		db = createTestDb();
		const row = db.prepare("SELECT * FROM metadata WHERE key = 'created_at'").get() as {
			key: string;
			value: string;
			type: string;
		};
		expect(row).toBeDefined();
		expect(row.type).toBe('string');
	});

	it('should be safe to call initializeSchema multiple times', () => {
		db = createTestDb();
		expect(() => initializeSchema(db)).not.toThrow();
	});

	it('should create the media_types table with seed data', () => {
		db = createTestDb();
		const rows = db.prepare('SELECT * FROM media_types ORDER BY id ASC').all() as { id: string; label: string }[];
		expect(rows).toHaveLength(3);
		expect(rows.map((r) => r.id)).toEqual(['audio', 'image', 'video']);
	});

	it('should create the categories table with video categories', () => {
		db = createTestDb();
		const rows = db.prepare("SELECT * FROM categories WHERE media_type_id = 'video' ORDER BY id ASC").all() as { id: string; label: string }[];
		expect(rows).toHaveLength(4);
		expect(rows.map((r) => r.id).sort()).toEqual(['movies', 'tv', 'video-uncategorized', 'youtube']);
	});

	it('should create the categories table with audio categories', () => {
		db = createTestDb();
		const rows = db.prepare("SELECT * FROM categories WHERE media_type_id = 'audio' ORDER BY id ASC").all() as { id: string; label: string }[];
		expect(rows).toHaveLength(3);
		expect(rows.map((r) => r.id).sort()).toEqual(['audio-uncategorized', 'music', 'podcast']);
	});

	it('should create the categories table with image categories', () => {
		db = createTestDb();
		const rows = db.prepare("SELECT * FROM categories WHERE media_type_id = 'image' ORDER BY id ASC").all() as { id: string; label: string }[];
		expect(rows).toHaveLength(3);
		expect(rows.map((r) => r.id).sort()).toEqual(['image-uncategorized', 'memes', 'photos']);
	});
});
