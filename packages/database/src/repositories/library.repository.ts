import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { LibraryRow } from '../types.js';

export class LibraryRepository {
	private stmts: {
		get: Statement<[string], LibraryRow>;
		getAll: Statement<[], LibraryRow>;
		insert: Statement<
			[{
				id: string;
				name: string;
				path: string;
				media_types: string;
				date_added: number;
			}]
		>;
		update: Statement<
			[{
				id: string;
				name: string;
				path: string;
				media_types: string;
			}]
		>;
		delete: Statement<[string]>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			get: db.prepare('SELECT * FROM libraries WHERE id = ?'),
			getAll: db.prepare('SELECT * FROM libraries ORDER BY date_added DESC'),
			insert: db.prepare(`
				INSERT INTO libraries (id, name, path, media_types, date_added)
				VALUES (@id, @name, @path, @media_types, @date_added)
			`),
			update: db.prepare(`
				UPDATE libraries SET
					name = @name, path = @path, media_types = @media_types
				WHERE id = @id
			`),
			delete: db.prepare('DELETE FROM libraries WHERE id = ?')
		};
	}

	get(id: string): LibraryRow | null {
		return this.stmts.get.get(id) ?? null;
	}

	getAll(): LibraryRow[] {
		return this.stmts.getAll.all();
	}

	insert(row: Omit<LibraryRow, 'created_at' | 'updated_at'>): void {
		this.stmts.insert.run(row);
	}

	update(id: string, updates: { name: string; path: string; media_types: string }): void {
		this.stmts.update.run({ id, ...updates });
	}

	delete(id: string): boolean {
		const result = this.stmts.delete.run(id);
		return result.changes > 0;
	}
}
