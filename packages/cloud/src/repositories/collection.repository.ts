import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { CloudCollectionRow } from '../types.js';

type InsertRow = Omit<CloudCollectionRow, 'created_at' | 'updated_at'>;

export class CloudCollectionRepository {
	private stmts: {
		get: Statement<[string], CloudCollectionRow>;
		getAll: Statement<[], CloudCollectionRow>;
		getByLibrary: Statement<[string], CloudCollectionRow>;
		insert: Statement<[InsertRow]>;
		update: Statement<[{ id: string; name: string; description: string | null; cover_path: string | null }]>;
		delete: Statement<[string]>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			get: db.prepare('SELECT * FROM collections WHERE id = ?'),
			getAll: db.prepare('SELECT * FROM collections ORDER BY created_at DESC'),
			getByLibrary: db.prepare('SELECT * FROM collections WHERE library_id = ? ORDER BY name ASC'),
			insert: db.prepare(`
				INSERT INTO collections (id, library_id, name, description, cover_path, kind)
				VALUES (@id, @library_id, @name, @description, @cover_path, @kind)
			`),
			update: db.prepare(`
				UPDATE collections SET
					name = @name, description = @description, cover_path = @cover_path
				WHERE id = @id
			`),
			delete: db.prepare('DELETE FROM collections WHERE id = ?')
		};
	}

	get(id: string): CloudCollectionRow | null {
		return this.stmts.get.get(id) ?? null;
	}

	getAll(): CloudCollectionRow[] {
		return this.stmts.getAll.all();
	}

	getByLibrary(libraryId: string): CloudCollectionRow[] {
		return this.stmts.getByLibrary.all(libraryId);
	}

	insert(row: InsertRow): void {
		this.stmts.insert.run(row);
	}

	update(id: string, updates: { name: string; description: string | null; cover_path: string | null }): void {
		this.stmts.update.run({ id, ...updates });
	}

	delete(id: string): boolean {
		const result = this.stmts.delete.run(id);
		return result.changes > 0;
	}
}
