import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { CollectionItemRow } from '../types.js';

type InsertRow = Omit<CollectionItemRow, 'created_at'>;

export class CollectionItemRepository {
	private stmts: {
		getByCollection: Statement<[string], CollectionItemRow>;
		insert: Statement<[InsertRow]>;
		delete: Statement<[string]>;
		deleteByCollection: Statement<[string]>;
		updatePosition: Statement<[{ id: string; position: number }]>;
	};

	private insertManyTx: ReturnType<DatabaseType['transaction']>;

	constructor(private db: DatabaseType) {
		this.stmts = {
			getByCollection: db.prepare(
				'SELECT * FROM collection_items WHERE collection_id = ? ORDER BY position ASC'
			),
			insert: db.prepare(`
				INSERT INTO collection_items (id, collection_id, item_id, position)
				VALUES (@id, @collection_id, @item_id, @position)
				ON CONFLICT(collection_id, item_id) DO UPDATE SET position = @position
			`),
			delete: db.prepare('DELETE FROM collection_items WHERE id = ?'),
			deleteByCollection: db.prepare('DELETE FROM collection_items WHERE collection_id = ?'),
			updatePosition: db.prepare(
				'UPDATE collection_items SET position = @position WHERE id = @id'
			)
		};

		this.insertManyTx = db.transaction((rows: InsertRow[]) => {
			for (const row of rows) {
				this.stmts.insert.run(row);
			}
		});
	}

	getByCollection(collectionId: string): CollectionItemRow[] {
		return this.stmts.getByCollection.all(collectionId);
	}

	insert(row: InsertRow): void {
		this.stmts.insert.run(row);
	}

	insertMany(rows: InsertRow[]): void {
		this.insertManyTx(rows);
	}

	delete(id: string): boolean {
		const result = this.stmts.delete.run(id);
		return result.changes > 0;
	}

	deleteByCollection(collectionId: string): number {
		const result = this.stmts.deleteByCollection.run(collectionId);
		return result.changes;
	}

	updatePosition(id: string, position: number): void {
		this.stmts.updatePosition.run({ id, position });
	}
}
