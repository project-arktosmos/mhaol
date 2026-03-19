import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { CloudSettingRow } from '../types.js';

export class CloudSettingsRepository {
	private stmts: {
		get: Statement<[string], CloudSettingRow>;
		getAll: Statement<[], CloudSettingRow>;
		getByPrefix: Statement<[string], CloudSettingRow>;
		set: Statement<[{ key: string; value: string }]>;
		delete: Statement<[string]>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			get: db.prepare('SELECT * FROM cloud_settings WHERE key = ?'),
			getAll: db.prepare('SELECT * FROM cloud_settings ORDER BY key'),
			getByPrefix: db.prepare('SELECT * FROM cloud_settings WHERE key LIKE ? ORDER BY key'),
			set: db.prepare(`
				INSERT INTO cloud_settings (key, value) VALUES (@key, @value)
				ON CONFLICT(key) DO UPDATE SET value = @value
			`),
			delete: db.prepare('DELETE FROM cloud_settings WHERE key = ?')
		};
	}

	get(key: string): string | null {
		const row = this.stmts.get.get(key);
		return row?.value ?? null;
	}

	getRow(key: string): CloudSettingRow | null {
		return this.stmts.get.get(key) ?? null;
	}

	getAll(): CloudSettingRow[] {
		return this.stmts.getAll.all();
	}

	getByPrefix(prefix: string): CloudSettingRow[] {
		return this.stmts.getByPrefix.all(prefix + '%');
	}

	set(key: string, value: string): void {
		this.stmts.set.run({ key, value });
	}

	delete(key: string): boolean {
		const result = this.stmts.delete.run(key);
		return result.changes > 0;
	}

	setMany(entries: Record<string, string>): void {
		const transaction = this.db.transaction((pairs: [string, string][]) => {
			for (const [key, value] of pairs) {
				this.stmts.set.run({ key, value });
			}
		});
		transaction(Object.entries(entries));
	}
}
