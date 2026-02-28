import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { MediaTypeRow } from '../types.js';

export class MediaTypeRepository {
	private stmts: {
		get: Statement<[string], MediaTypeRow>;
		getAll: Statement<[], MediaTypeRow>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			get: db.prepare('SELECT * FROM media_types WHERE id = ?'),
			getAll: db.prepare('SELECT * FROM media_types ORDER BY id ASC')
		};
	}

	get(id: string): MediaTypeRow | null {
		return this.stmts.get.get(id) ?? null;
	}

	getAll(): MediaTypeRow[] {
		return this.stmts.getAll.all();
	}
}
