import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { MediaListLinkRow } from '../types.js';

type InsertRow = Omit<MediaListLinkRow, 'created_at'>;

export class MediaListLinkRepository {
	private stmts: {
		getByList: Statement<[string], MediaListLinkRow>;
		getByListAndService: Statement<[string, string], MediaListLinkRow>;
		upsert: Statement<[InsertRow]>;
		delete: Statement<[string, string]>;
		deleteByList: Statement<[string]>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			getByList: db.prepare(
				'SELECT * FROM media_list_links WHERE list_id = ? ORDER BY service ASC'
			),
			getByListAndService: db.prepare(
				'SELECT * FROM media_list_links WHERE list_id = ? AND service = ?'
			),
			upsert: db.prepare(`
				INSERT INTO media_list_links (id, list_id, service, service_id, season_number)
				VALUES (@id, @list_id, @service, @service_id, @season_number)
				ON CONFLICT(list_id, service) DO UPDATE SET
					service_id = excluded.service_id,
					season_number = excluded.season_number
			`),
			delete: db.prepare(
				'DELETE FROM media_list_links WHERE list_id = ? AND service = ?'
			),
			deleteByList: db.prepare(
				'DELETE FROM media_list_links WHERE list_id = ?'
			)
		};
	}

	getByList(listId: string): MediaListLinkRow[] {
		return this.stmts.getByList.all(listId);
	}

	getByListAndService(listId: string, service: string): MediaListLinkRow | null {
		return this.stmts.getByListAndService.get(listId, service) ?? null;
	}

	upsert(row: InsertRow): void {
		this.stmts.upsert.run(row);
	}

	delete(listId: string, service: string): boolean {
		const result = this.stmts.delete.run(listId, service);
		return result.changes > 0;
	}

	deleteByList(listId: string): number {
		const result = this.stmts.deleteByList.run(listId);
		return result.changes;
	}
}
