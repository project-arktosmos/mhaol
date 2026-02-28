import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { LibraryItemRow } from '../types.js';

type InsertRow = Omit<LibraryItemRow, 'created_at' | 'updated_at'>;

export class LibraryItemRepository {
	private stmts: {
		get: Statement<[string], LibraryItemRow>;
		getByLibrary: Statement<[string], LibraryItemRow>;
		getByMediaType: Statement<[string], LibraryItemRow>;
		insert: Statement<[InsertRow]>;
		delete: Statement<[string]>;
		deleteByLibrary: Statement<[string]>;
		existsByPath: Statement<[string], { id: string }>;
		updateCategory: Statement<[{ id: string; category_id: string }]>;
		clearCategory: Statement<[string]>;
		getByCategory: Statement<[string], LibraryItemRow>;
		getUncategorizedByMediaType: Statement<[string], LibraryItemRow>;
		updateMediaType: Statement<[{ id: string; media_type: string }]>;
	};

	private insertManyTx: ReturnType<DatabaseType['transaction']>;
	private syncLibraryTx: ReturnType<DatabaseType['transaction']>;

	constructor(private db: DatabaseType) {
		this.stmts = {
			get: db.prepare('SELECT * FROM library_items WHERE id = ?'),
			getByLibrary: db.prepare('SELECT * FROM library_items WHERE library_id = ? ORDER BY path ASC'),
			getByMediaType: db.prepare('SELECT * FROM library_items WHERE media_type = ? ORDER BY path ASC'),
			insert: db.prepare(`
				INSERT INTO library_items (id, library_id, path, extension, media_type, category_id)
				VALUES (@id, @library_id, @path, @extension, @media_type, @category_id)
			`),
			delete: db.prepare('DELETE FROM library_items WHERE id = ?'),
			deleteByLibrary: db.prepare('DELETE FROM library_items WHERE library_id = ?'),
			existsByPath: db.prepare('SELECT id FROM library_items WHERE path = ?'),
			updateCategory: db.prepare(`
				UPDATE library_items SET category_id = @category_id WHERE id = @id
			`),
			clearCategory: db.prepare(`
				UPDATE library_items SET category_id = NULL WHERE id = ?
			`),
			getByCategory: db.prepare('SELECT * FROM library_items WHERE category_id = ? ORDER BY path ASC'),
			getUncategorizedByMediaType: db.prepare('SELECT * FROM library_items WHERE media_type = ? AND category_id IS NULL ORDER BY path ASC'),
			updateMediaType: db.prepare(`
				UPDATE library_items SET media_type = @media_type WHERE id = @id
			`)
		};

		this.insertManyTx = db.transaction((rows: InsertRow[]) => {
			for (const row of rows) {
				this.stmts.insert.run(row);
			}
		});

		this.syncLibraryTx = db.transaction((libraryId: string, newFiles: InsertRow[]) => {
			const existing = this.stmts.getByLibrary.all(libraryId);
			const scannedPaths = new Set(newFiles.map((f) => f.path));
			const existingPaths = new Set(existing.map((e) => e.path));

			// Remove items whose files no longer exist on disk
			for (const item of existing) {
				if (!scannedPaths.has(item.path)) {
					this.stmts.delete.run(item.id);
				}
			}

			// Insert only genuinely new files
			for (const file of newFiles) {
				if (!existingPaths.has(file.path)) {
					this.stmts.insert.run(file);
				}
			}
		});
	}

	get(id: string): LibraryItemRow | null {
		return this.stmts.get.get(id) ?? null;
	}

	getByLibrary(libraryId: string): LibraryItemRow[] {
		return this.stmts.getByLibrary.all(libraryId);
	}

	getByMediaType(mediaType: string): LibraryItemRow[] {
		return this.stmts.getByMediaType.all(mediaType);
	}

	insert(row: InsertRow): void {
		this.stmts.insert.run(row);
	}

	insertMany(rows: InsertRow[]): void {
		this.insertManyTx(rows);
	}

	syncLibrary(libraryId: string, newFiles: InsertRow[]): void {
		this.syncLibraryTx(libraryId, newFiles);
	}

	delete(id: string): boolean {
		const result = this.stmts.delete.run(id);
		return result.changes > 0;
	}

	deleteByLibrary(libraryId: string): number {
		const result = this.stmts.deleteByLibrary.run(libraryId);
		return result.changes;
	}

	existsByPath(path: string): string | null {
		const row = this.stmts.existsByPath.get(path);
		return row ? row.id : null;
	}

	updateCategory(id: string, categoryId: string): void {
		this.stmts.updateCategory.run({ id, category_id: categoryId });
	}

	clearCategory(id: string): void {
		this.stmts.clearCategory.run(id);
	}

	getByCategory(categoryId: string): LibraryItemRow[] {
		return this.stmts.getByCategory.all(categoryId);
	}

	getUncategorizedByMediaType(mediaType: string): LibraryItemRow[] {
		return this.stmts.getUncategorizedByMediaType.all(mediaType);
	}

	updateMediaType(id: string, mediaType: string): void {
		this.stmts.updateMediaType.run({ id, media_type: mediaType });
	}
}
