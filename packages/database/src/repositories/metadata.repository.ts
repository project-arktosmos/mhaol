import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { MetadataRow } from '../types.js';

export class MetadataRepository {
	private stmts: {
		get: Statement<[string], MetadataRow>;
		getAll: Statement<[], MetadataRow>;
		getByType: Statement<[string], MetadataRow>;
		set: Statement<[{ key: string; value: string; type: string }]>;
		delete: Statement<[string]>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			get: db.prepare('SELECT * FROM metadata WHERE key = ?'),
			getAll: db.prepare('SELECT * FROM metadata ORDER BY key'),
			getByType: db.prepare('SELECT * FROM metadata WHERE type = ? ORDER BY key'),
			set: db.prepare(`
				INSERT INTO metadata (key, value, type) VALUES (@key, @value, @type)
				ON CONFLICT(key) DO UPDATE SET value = @value, type = @type
			`),
			delete: db.prepare('DELETE FROM metadata WHERE key = ?')
		};
	}

	get(key: string): MetadataRow | null {
		return this.stmts.get.get(key) ?? null;
	}

	getValue<T = string>(key: string): T | null {
		const row = this.get(key);
		if (!row) return null;
		return this.parseValue(row) as T;
	}

	getAll(): MetadataRow[] {
		return this.stmts.getAll.all();
	}

	getByType(type: MetadataRow['type']): MetadataRow[] {
		return this.stmts.getByType.all(type);
	}

	set(key: string, value: string | number | boolean | object): void {
		const { serialized, type } = this.serializeValue(value);
		this.stmts.set.run({ key, value: serialized, type });
	}

	delete(key: string): boolean {
		const result = this.stmts.delete.run(key);
		return result.changes > 0;
	}

	private serializeValue(value: string | number | boolean | object): {
		serialized: string;
		type: MetadataRow['type'];
	} {
		if (typeof value === 'string') return { serialized: value, type: 'string' };
		if (typeof value === 'number') return { serialized: String(value), type: 'number' };
		if (typeof value === 'boolean') return { serialized: String(value), type: 'boolean' };
		return { serialized: JSON.stringify(value), type: 'json' };
	}

	private parseValue(row: MetadataRow): string | number | boolean | unknown {
		switch (row.type) {
			case 'number':
				return Number(row.value);
			case 'boolean':
				return row.value === 'true';
			case 'json':
				return JSON.parse(row.value);
			default:
				return row.value;
		}
	}
}
