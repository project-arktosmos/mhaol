import Database from 'better-sqlite3';
import type { Database as DatabaseType } from 'better-sqlite3';
import type { DatabaseConfig } from './types.js';
import { DEFAULT_DB_PATH } from './utils/path.js';
import { initializeSchema } from './schema.js';

let db: DatabaseType | null = null;

export function getDatabase(config?: DatabaseConfig): DatabaseType {
	if (db) return db;

	const dbPath = config?.dbPath ?? DEFAULT_DB_PATH;
	const walMode = config?.walMode ?? true;

	db = new Database(dbPath);

	db.pragma('foreign_keys = ON');
	db.pragma('busy_timeout = 5000');

	if (walMode) {
		db.pragma('journal_mode = WAL');
	}

	initializeSchema(db);

	return db;
}

export function closeDatabase(): void {
	if (db) {
		db.close();
		db = null;
	}
}

export function isDatabaseOpen(): boolean {
	return db !== null;
}
