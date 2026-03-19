import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { CloudLibraryRow } from '../types.js';

type InsertRow = Omit<CloudLibraryRow, 'created_at' | 'updated_at' | 'scan_status' | 'scan_error' | 'item_count'>;

export class CloudLibraryRepository {
	private stmts: {
		get: Statement<[string], CloudLibraryRow>;
		getAll: Statement<[], CloudLibraryRow>;
		insert: Statement<[InsertRow]>;
		update: Statement<[{ id: string; name: string; path: string }]>;
		updateScanStatus: Statement<[{ id: string; scan_status: string; scan_error: string | null }]>;
		updateItemCount: Statement<[{ id: string; item_count: number }]>;
		delete: Statement<[string]>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			get: db.prepare('SELECT * FROM libraries WHERE id = ?'),
			getAll: db.prepare('SELECT * FROM libraries ORDER BY created_at DESC'),
			insert: db.prepare(`
				INSERT INTO libraries (id, name, path, kind)
				VALUES (@id, @name, @path, @kind)
			`),
			update: db.prepare(`
				UPDATE libraries SET name = @name, path = @path WHERE id = @id
			`),
			updateScanStatus: db.prepare(`
				UPDATE libraries SET scan_status = @scan_status, scan_error = @scan_error WHERE id = @id
			`),
			updateItemCount: db.prepare(`
				UPDATE libraries SET item_count = @item_count WHERE id = @id
			`),
			delete: db.prepare('DELETE FROM libraries WHERE id = ?')
		};
	}

	get(id: string): CloudLibraryRow | null {
		return this.stmts.get.get(id) ?? null;
	}

	getAll(): CloudLibraryRow[] {
		return this.stmts.getAll.all();
	}

	insert(row: InsertRow): void {
		this.stmts.insert.run(row);
	}

	update(id: string, updates: { name: string; path: string }): void {
		this.stmts.update.run({ id, ...updates });
	}

	updateScanStatus(id: string, status: string, error?: string | null): void {
		this.stmts.updateScanStatus.run({ id, scan_status: status, scan_error: error ?? null });
	}

	updateItemCount(id: string, count: number): void {
		this.stmts.updateItemCount.run({ id, item_count: count });
	}

	delete(id: string): boolean {
		const result = this.stmts.delete.run(id);
		return result.changes > 0;
	}
}
