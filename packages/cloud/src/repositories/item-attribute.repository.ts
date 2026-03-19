import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { ItemAttributeRow } from '../types.js';

type UpsertRow = Omit<ItemAttributeRow, 'created_at' | 'updated_at'>;

export class ItemAttributeRepository {
	private stmts: {
		getByItem: Statement<[string], ItemAttributeRow>;
		getByItemAndKey: Statement<[{ item_id: string; key: string }], ItemAttributeRow>;
		getByKey: Statement<[string], ItemAttributeRow>;
		getByKeyAndValue: Statement<[{ key: string; value: string }], ItemAttributeRow>;
		search: Statement<[{ key: string; pattern: string }], ItemAttributeRow>;
		set: Statement<[UpsertRow]>;
		delete: Statement<[string]>;
		deleteByItem: Statement<[string]>;
		deleteByItemAndKey: Statement<[{ item_id: string; key: string }]>;
		distinctKeys: Statement<[], { key: string }>;
		distinctValues: Statement<[string], { value: string }>;
	};

	private setManyTx: ReturnType<DatabaseType['transaction']>;

	constructor(private db: DatabaseType) {
		this.stmts = {
			getByItem: db.prepare(
				'SELECT * FROM item_attributes WHERE item_id = ? ORDER BY key ASC'
			),
			getByItemAndKey: db.prepare(
				'SELECT * FROM item_attributes WHERE item_id = @item_id AND key = @key ORDER BY source ASC'
			),
			getByKey: db.prepare(
				'SELECT * FROM item_attributes WHERE key = ? ORDER BY value ASC'
			),
			getByKeyAndValue: db.prepare(
				'SELECT * FROM item_attributes WHERE key = @key AND value = @value ORDER BY item_id ASC'
			),
			search: db.prepare(
				'SELECT * FROM item_attributes WHERE key = @key AND value LIKE @pattern ORDER BY value ASC'
			),
			set: db.prepare(`
				INSERT INTO item_attributes (id, item_id, key, value, attribute_type_id, source, confidence)
				VALUES (@id, @item_id, @key, @value, @attribute_type_id, @source, @confidence)
				ON CONFLICT(item_id, key, source) DO UPDATE SET
					value = @value,
					attribute_type_id = @attribute_type_id,
					confidence = @confidence
			`),
			delete: db.prepare('DELETE FROM item_attributes WHERE id = ?'),
			deleteByItem: db.prepare('DELETE FROM item_attributes WHERE item_id = ?'),
			deleteByItemAndKey: db.prepare(
				'DELETE FROM item_attributes WHERE item_id = @item_id AND key = @key'
			),
			distinctKeys: db.prepare(
				'SELECT DISTINCT key FROM item_attributes ORDER BY key ASC'
			),
			distinctValues: db.prepare(
				'SELECT DISTINCT value FROM item_attributes WHERE key = ? ORDER BY value ASC'
			)
		};

		this.setManyTx = db.transaction((rows: UpsertRow[]) => {
			for (const row of rows) {
				this.stmts.set.run(row);
			}
		});
	}

	getByItem(itemId: string): ItemAttributeRow[] {
		return this.stmts.getByItem.all(itemId);
	}

	getByItemAndKey(itemId: string, key: string): ItemAttributeRow[] {
		return this.stmts.getByItemAndKey.all({ item_id: itemId, key });
	}

	getByKey(key: string): ItemAttributeRow[] {
		return this.stmts.getByKey.all(key);
	}

	getByKeyAndValue(key: string, value: string): ItemAttributeRow[] {
		return this.stmts.getByKeyAndValue.all({ key, value });
	}

	search(key: string, valuePattern: string): ItemAttributeRow[] {
		return this.stmts.search.all({ key, pattern: '%' + valuePattern + '%' });
	}

	set(row: UpsertRow): void {
		this.stmts.set.run(row);
	}

	setMany(rows: UpsertRow[]): void {
		this.setManyTx(rows);
	}

	delete(id: string): boolean {
		const result = this.stmts.delete.run(id);
		return result.changes > 0;
	}

	deleteByItem(itemId: string): number {
		const result = this.stmts.deleteByItem.run(itemId);
		return result.changes;
	}

	deleteByItemAndKey(itemId: string, key: string): number {
		const result = this.stmts.deleteByItemAndKey.run({ item_id: itemId, key });
		return result.changes;
	}

	getDistinctKeys(): string[] {
		return this.stmts.distinctKeys.all().map((r) => r.key);
	}

	getDistinctValues(key: string): string[] {
		return this.stmts.distinctValues.all(key).map((r) => r.value);
	}
}
