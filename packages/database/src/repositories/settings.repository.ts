import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { SettingRow } from '../types.js';

export class SettingsRepository {
	private stmts: {
		get: Statement<[string], SettingRow>;
		getAll: Statement<[], SettingRow>;
		getByPrefix: Statement<[string], SettingRow>;
		set: Statement<[{ key: string; value: string }]>;
		delete: Statement<[string]>;
		exists: Statement<[string]>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			get: db.prepare('SELECT * FROM settings WHERE key = ?'),
			getAll: db.prepare('SELECT * FROM settings ORDER BY key'),
			getByPrefix: db.prepare('SELECT * FROM settings WHERE key LIKE ? ORDER BY key'),
			set: db.prepare(`
				INSERT INTO settings (key, value) VALUES (@key, @value)
				ON CONFLICT(key) DO UPDATE SET value = @value
			`),
			delete: db.prepare('DELETE FROM settings WHERE key = ?'),
			exists: db.prepare('SELECT 1 FROM settings WHERE key = ?')
		};
	}

	get(key: string): string | null {
		const row = this.stmts.get.get(key);
		return row?.value ?? null;
	}

	getRow(key: string): SettingRow | null {
		return this.stmts.get.get(key) ?? null;
	}

	getAll(): SettingRow[] {
		return this.stmts.getAll.all();
	}

	getByPrefix(prefix: string): SettingRow[] {
		return this.stmts.getByPrefix.all(prefix + '%');
	}

	set(key: string, value: string): void {
		this.stmts.set.run({ key, value });
	}

	delete(key: string): boolean {
		const result = this.stmts.delete.run(key);
		return result.changes > 0;
	}

	exists(key: string): boolean {
		return this.stmts.exists.get(key) !== undefined;
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
