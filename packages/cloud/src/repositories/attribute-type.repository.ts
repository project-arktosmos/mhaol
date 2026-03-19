import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { AttributeTypeRow } from '../types.js';

export class AttributeTypeRepository {
	private stmts: {
		get: Statement<[string], AttributeTypeRow>;
		getAll: Statement<[], AttributeTypeRow>;
		insert: Statement<[{ id: string; label: string }]>;
		delete: Statement<[string]>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			get: db.prepare('SELECT * FROM attribute_types WHERE id = ?'),
			getAll: db.prepare('SELECT * FROM attribute_types ORDER BY label'),
			insert: db.prepare(`
				INSERT OR IGNORE INTO attribute_types (id, label) VALUES (@id, @label)
			`),
			delete: db.prepare('DELETE FROM attribute_types WHERE id = ?')
		};
	}

	get(id: string): AttributeTypeRow | null {
		return this.stmts.get.get(id) ?? null;
	}

	getAll(): AttributeTypeRow[] {
		return this.stmts.getAll.all();
	}

	insert(id: string, label: string): void {
		this.stmts.insert.run({ id, label });
	}

	delete(id: string): boolean {
		const result = this.stmts.delete.run(id);
		return result.changes > 0;
	}
}
