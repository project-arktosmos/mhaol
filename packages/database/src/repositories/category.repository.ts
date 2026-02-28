import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { CategoryRow } from '../types.js';

export class CategoryRepository {
	private stmts: {
		get: Statement<[string], CategoryRow>;
		getAll: Statement<[], CategoryRow>;
		getByMediaType: Statement<[string], CategoryRow>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			get: db.prepare('SELECT * FROM categories WHERE id = ?'),
			getAll: db.prepare('SELECT * FROM categories ORDER BY media_type_id ASC, label ASC'),
			getByMediaType: db.prepare('SELECT * FROM categories WHERE media_type_id = ? ORDER BY label ASC')
		};
	}

	get(id: string): CategoryRow | null {
		return this.stmts.get.get(id) ?? null;
	}

	getAll(): CategoryRow[] {
		return this.stmts.getAll.all();
	}

	getByMediaType(mediaTypeId: string): CategoryRow[] {
		return this.stmts.getByMediaType.all(mediaTypeId);
	}
}
