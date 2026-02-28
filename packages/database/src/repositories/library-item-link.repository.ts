import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { LibraryItemLinkRow } from '../types.js';

type InsertRow = Omit<LibraryItemLinkRow, 'created_at'>;

export class LibraryItemLinkRepository {
	private stmts: {
		getByItem: Statement<[string], LibraryItemLinkRow>;
		getByItemAndService: Statement<[string, string], LibraryItemLinkRow>;
		getByServiceId: Statement<[string, string], LibraryItemLinkRow>;
		upsert: Statement<[InsertRow]>;
		delete: Statement<[string, string]>;
		deleteByItem: Statement<[string]>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			getByItem: db.prepare(
				'SELECT * FROM library_item_links WHERE library_item_id = ? ORDER BY service ASC'
			),
			getByItemAndService: db.prepare(
				'SELECT * FROM library_item_links WHERE library_item_id = ? AND service = ?'
			),
			getByServiceId: db.prepare(
				'SELECT * FROM library_item_links WHERE service = ? AND service_id = ? ORDER BY library_item_id ASC'
			),
			upsert: db.prepare(`
				INSERT INTO library_item_links (id, library_item_id, service, service_id, season_number, episode_number)
				VALUES (@id, @library_item_id, @service, @service_id, @season_number, @episode_number)
				ON CONFLICT(library_item_id, service) DO UPDATE SET
					service_id = excluded.service_id,
					season_number = excluded.season_number,
					episode_number = excluded.episode_number
			`),
			delete: db.prepare(
				'DELETE FROM library_item_links WHERE library_item_id = ? AND service = ?'
			),
			deleteByItem: db.prepare(
				'DELETE FROM library_item_links WHERE library_item_id = ?'
			)
		};
	}

	getByItem(libraryItemId: string): LibraryItemLinkRow[] {
		return this.stmts.getByItem.all(libraryItemId);
	}

	getByItemAndService(libraryItemId: string, service: string): LibraryItemLinkRow | null {
		return this.stmts.getByItemAndService.get(libraryItemId, service) ?? null;
	}

	getByServiceId(service: string, serviceId: string): LibraryItemLinkRow[] {
		return this.stmts.getByServiceId.all(service, serviceId);
	}

	upsert(row: InsertRow): void {
		this.stmts.upsert.run(row);
	}

	delete(libraryItemId: string, service: string): boolean {
		const result = this.stmts.delete.run(libraryItemId, service);
		return result.changes > 0;
	}

	deleteByItem(libraryItemId: string): number {
		const result = this.stmts.deleteByItem.run(libraryItemId);
		return result.changes;
	}
}
