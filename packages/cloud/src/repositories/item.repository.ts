import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { CloudItemRow } from '../types.js';

type InsertRow = Omit<CloudItemRow, 'created_at' | 'updated_at'>;

export class CloudItemRepository {
	private stmts: {
		get: Statement<[string], CloudItemRow>;
		getByLibrary: Statement<[string], CloudItemRow>;
		getByExtension: Statement<[string], CloudItemRow>;
		insert: Statement<[InsertRow]>;
		delete: Statement<[string]>;
		deleteByLibrary: Statement<[string]>;
		existsByPath: Statement<[{ library_id: string; path: string }], { id: string }>;
		search: Statement<[string], CloudItemRow>;
	};

	private insertManyTx: ReturnType<DatabaseType['transaction']>;
	private syncLibraryTx: ReturnType<DatabaseType['transaction']>;

	constructor(private db: DatabaseType) {
		this.stmts = {
			get: db.prepare('SELECT * FROM items WHERE id = ?'),
			getByLibrary: db.prepare('SELECT * FROM items WHERE library_id = ? ORDER BY path ASC'),
			getByExtension: db.prepare('SELECT * FROM items WHERE extension = ? ORDER BY path ASC'),
			insert: db.prepare(`
				INSERT INTO items (id, library_id, path, filename, extension, size_bytes, mime_type, checksum)
				VALUES (@id, @library_id, @path, @filename, @extension, @size_bytes, @mime_type, @checksum)
			`),
			delete: db.prepare('DELETE FROM items WHERE id = ?'),
			deleteByLibrary: db.prepare('DELETE FROM items WHERE library_id = ?'),
			existsByPath: db.prepare('SELECT id FROM items WHERE library_id = @library_id AND path = @path'),
			search: db.prepare('SELECT * FROM items WHERE filename LIKE ? ORDER BY filename ASC')
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

			for (const item of existing) {
				if (!scannedPaths.has(item.path)) {
					this.stmts.delete.run(item.id);
				}
			}

			for (const file of newFiles) {
				if (!existingPaths.has(file.path)) {
					this.stmts.insert.run(file);
				}
			}
		});
	}

	get(id: string): CloudItemRow | null {
		return this.stmts.get.get(id) ?? null;
	}

	getByLibrary(libraryId: string): CloudItemRow[] {
		return this.stmts.getByLibrary.all(libraryId);
	}

	getByExtension(extension: string): CloudItemRow[] {
		return this.stmts.getByExtension.all(extension);
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

	existsByPath(libraryId: string, path: string): string | null {
		const row = this.stmts.existsByPath.get({ library_id: libraryId, path });
		return row ? row.id : null;
	}

	search(query: string): CloudItemRow[] {
		return this.stmts.search.all('%' + query + '%');
	}
}
