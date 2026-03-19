import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { ItemLinkRow } from '../types.js';

type InsertRow = Omit<ItemLinkRow, 'created_at'>;

export class ItemLinkRepository {
	private stmts: {
		getByItem: Statement<[string], ItemLinkRow>;
		getByService: Statement<[{ service: string; service_id: string }], ItemLinkRow>;
		insert: Statement<[InsertRow]>;
		delete: Statement<[string]>;
		deleteByItem: Statement<[string]>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			getByItem: db.prepare('SELECT * FROM item_links WHERE item_id = ? ORDER BY service ASC'),
			getByService: db.prepare(
				'SELECT * FROM item_links WHERE service = @service AND service_id = @service_id'
			),
			insert: db.prepare(`
				INSERT INTO item_links (id, item_id, service, service_id, extra)
				VALUES (@id, @item_id, @service, @service_id, @extra)
				ON CONFLICT(item_id, service) DO UPDATE SET
					service_id = @service_id, extra = @extra
			`),
			delete: db.prepare('DELETE FROM item_links WHERE id = ?'),
			deleteByItem: db.prepare('DELETE FROM item_links WHERE item_id = ?')
		};
	}

	getByItem(itemId: string): ItemLinkRow[] {
		return this.stmts.getByItem.all(itemId);
	}

	getByService(service: string, serviceId: string): ItemLinkRow[] {
		return this.stmts.getByService.all({ service, service_id: serviceId });
	}

	set(row: InsertRow): void {
		this.stmts.insert.run(row);
	}

	delete(id: string): boolean {
		const result = this.stmts.delete.run(id);
		return result.changes > 0;
	}

	deleteByItem(itemId: string): number {
		const result = this.stmts.deleteByItem.run(itemId);
		return result.changes;
	}
}
