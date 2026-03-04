import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { MediaListItemRow } from '../types.js';

type InsertRow = Omit<MediaListItemRow, 'created_at'>;

export class MediaListItemRepository {
	private stmts: {
		getByList: Statement<[string], MediaListItemRow>;
		getByItem: Statement<[string], MediaListItemRow>;
		insert: Statement<[InsertRow]>;
		delete: Statement<[string]>;
		deleteByList: Statement<[string]>;
		deleteByItem: Statement<[string]>;
	};

	private syncListTx: ReturnType<DatabaseType['transaction']>;

	constructor(private db: DatabaseType) {
		this.stmts = {
			getByList: db.prepare('SELECT * FROM media_list_items WHERE list_id = ? ORDER BY position ASC'),
			getByItem: db.prepare('SELECT * FROM media_list_items WHERE library_item_id = ?'),
			insert: db.prepare(`
				INSERT OR IGNORE INTO media_list_items (id, list_id, library_item_id, position)
				VALUES (@id, @list_id, @library_item_id, @position)
			`),
			delete: db.prepare('DELETE FROM media_list_items WHERE id = ?'),
			deleteByList: db.prepare('DELETE FROM media_list_items WHERE list_id = ?'),
			deleteByItem: db.prepare('DELETE FROM media_list_items WHERE library_item_id = ?')
		};

		this.syncListTx = db.transaction((listId: string, items: Array<{ id: string; library_item_id: string; position: number }>) => {
			this.stmts.deleteByList.run(listId);
			for (const item of items) {
				this.stmts.insert.run({
					id: item.id,
					list_id: listId,
					library_item_id: item.library_item_id,
					position: item.position
				});
			}
		});
	}

	getByList(listId: string): MediaListItemRow[] {
		return this.stmts.getByList.all(listId);
	}

	getByItem(libraryItemId: string): MediaListItemRow[] {
		return this.stmts.getByItem.all(libraryItemId);
	}

	insert(row: InsertRow): void {
		this.stmts.insert.run(row);
	}

	delete(id: string): boolean {
		const result = this.stmts.delete.run(id);
		return result.changes > 0;
	}

	deleteByList(listId: string): number {
		const result = this.stmts.deleteByList.run(listId);
		return result.changes;
	}

	deleteByItem(libraryItemId: string): number {
		const result = this.stmts.deleteByItem.run(libraryItemId);
		return result.changes;
	}

	syncList(listId: string, items: Array<{ id: string; library_item_id: string; position: number }>): void {
		this.syncListTx(listId, items);
	}
}
