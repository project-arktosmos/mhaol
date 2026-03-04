import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { MediaListRow } from '../types.js';

type InsertRow = Omit<MediaListRow, 'created_at' | 'updated_at'>;

export class MediaListRepository {
	private stmts: {
		get: Statement<[string], MediaListRow>;
		getAll: Statement<[], MediaListRow>;
		getByLibrary: Statement<[string], MediaListRow>;
		getAutoByLibrary: Statement<[string], MediaListRow>;
		getBySourcePath: Statement<[string], MediaListRow>;
		insert: Statement<[InsertRow]>;
		updateTitle: Statement<[{ id: string; title: string }]>;
		delete: Statement<[string]>;
		deleteByLibrary: Statement<[string]>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			get: db.prepare('SELECT * FROM media_lists WHERE id = ?'),
			getAll: db.prepare('SELECT * FROM media_lists ORDER BY title ASC'),
			getByLibrary: db.prepare('SELECT * FROM media_lists WHERE library_id = ? ORDER BY title ASC'),
			getAutoByLibrary: db.prepare("SELECT * FROM media_lists WHERE library_id = ? AND source = 'auto' ORDER BY title ASC"),
			getBySourcePath: db.prepare('SELECT * FROM media_lists WHERE source_path = ?'),
			insert: db.prepare(`
				INSERT INTO media_lists (id, library_id, title, description, cover_image, media_type, source, source_path)
				VALUES (@id, @library_id, @title, @description, @cover_image, @media_type, @source, @source_path)
			`),
			updateTitle: db.prepare('UPDATE media_lists SET title = @title WHERE id = @id'),
			delete: db.prepare('DELETE FROM media_lists WHERE id = ?'),
			deleteByLibrary: db.prepare('DELETE FROM media_lists WHERE library_id = ?')
		};
	}

	get(id: string): MediaListRow | null {
		return this.stmts.get.get(id) ?? null;
	}

	getAll(): MediaListRow[] {
		return this.stmts.getAll.all();
	}

	getByLibrary(libraryId: string): MediaListRow[] {
		return this.stmts.getByLibrary.all(libraryId);
	}

	getAutoByLibrary(libraryId: string): MediaListRow[] {
		return this.stmts.getAutoByLibrary.all(libraryId);
	}

	getBySourcePath(sourcePath: string): MediaListRow | null {
		return this.stmts.getBySourcePath.get(sourcePath) ?? null;
	}

	insert(row: InsertRow): void {
		this.stmts.insert.run(row);
	}

	updateTitle(id: string, title: string): void {
		this.stmts.updateTitle.run({ id, title });
	}

	delete(id: string): boolean {
		const result = this.stmts.delete.run(id);
		return result.changes > 0;
	}

	deleteByLibrary(libraryId: string): number {
		const result = this.stmts.deleteByLibrary.run(libraryId);
		return result.changes;
	}
}
