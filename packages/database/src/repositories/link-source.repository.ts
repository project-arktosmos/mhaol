import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { LinkSourceRow } from '../types.js';

export class LinkSourceRepository {
	private stmts: {
		getAll: Statement<[], LinkSourceRow>;
		getByMediaType: Statement<[string], LinkSourceRow>;
		getByMediaTypeAndCategory: Statement<[string, string], LinkSourceRow>;
		upsert: Statement<[LinkSourceRow]>;
		deleteByPlugin: Statement<[string]>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			getAll: db.prepare('SELECT * FROM link_sources ORDER BY media_type_id, label ASC'),
			getByMediaType: db.prepare(
				'SELECT * FROM link_sources WHERE media_type_id = ? AND category_id IS NULL ORDER BY label ASC'
			),
			getByMediaTypeAndCategory: db.prepare(
				'SELECT * FROM link_sources WHERE media_type_id = ? AND (category_id IS NULL OR category_id = ?) ORDER BY label ASC'
			),
			upsert: db.prepare(`
				INSERT INTO link_sources (id, plugin, service, label, media_type_id, category_id)
				VALUES (@id, @plugin, @service, @label, @media_type_id, @category_id)
				ON CONFLICT(service, media_type_id, category_id) DO UPDATE SET
					plugin = excluded.plugin,
					label = excluded.label
			`),
			deleteByPlugin: db.prepare('DELETE FROM link_sources WHERE plugin = ?')
		};
	}

	getAll(): LinkSourceRow[] {
		return this.stmts.getAll.all();
	}

	getByMediaType(mediaTypeId: string): LinkSourceRow[] {
		return this.stmts.getByMediaType.all(mediaTypeId);
	}

	getByMediaTypeAndCategory(mediaTypeId: string, categoryId: string): LinkSourceRow[] {
		return this.stmts.getByMediaTypeAndCategory.all(mediaTypeId, categoryId);
	}

	upsert(row: LinkSourceRow): void {
		this.stmts.upsert.run(row);
	}

	deleteByPlugin(plugin: string): number {
		const result = this.stmts.deleteByPlugin.run(plugin);
		return result.changes;
	}
}
