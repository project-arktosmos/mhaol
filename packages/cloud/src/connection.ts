import Database from 'better-sqlite3';
import type { Database as DatabaseType } from 'better-sqlite3';
import type { CloudDatabaseConfig } from './types.js';
import { DEFAULT_CLOUD_DB_PATH } from './utils/path.js';
import { initializeCloudSchema } from './schema.js';

let db: DatabaseType | null = null;

export function getCloudDatabase(config?: CloudDatabaseConfig): DatabaseType {
	if (db) return db;

	const dbPath = config?.dbPath ?? DEFAULT_CLOUD_DB_PATH;
	const walMode = config?.walMode ?? true;

	db = new Database(dbPath);

	db.pragma('foreign_keys = ON');
	db.pragma('busy_timeout = 5000');

	if (walMode) {
		db.pragma('journal_mode = WAL');
	}

	initializeCloudSchema(db);

	return db;
}

export function closeCloudDatabase(): void {
	if (db) {
		db.close();
		db = null;
	}
}

export function isCloudDatabaseOpen(): boolean {
	return db !== null;
}
