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
		expect(row.value).toBe('2');
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
});
