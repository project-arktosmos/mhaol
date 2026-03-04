import type { Database as DatabaseType, Statement } from 'better-sqlite3';
import type { SignalingServerRow } from '../types.js';

type InsertRow = Omit<SignalingServerRow, 'created_at' | 'updated_at'>;
type UpdateRow = { id: string; name: string; url: string; enabled: number };

export class SignalingServerRepository {
	private stmts: {
		get: Statement<[string], SignalingServerRow>;
		getAll: Statement<[], SignalingServerRow>;
		getEnabled: Statement<[], SignalingServerRow>;
		insert: Statement<[InsertRow]>;
		update: Statement<[UpdateRow]>;
		delete: Statement<[string]>;
	};

	constructor(private db: DatabaseType) {
		this.stmts = {
			get: db.prepare('SELECT * FROM signaling_servers WHERE id = ?'),
			getAll: db.prepare('SELECT * FROM signaling_servers ORDER BY created_at ASC'),
			getEnabled: db.prepare('SELECT * FROM signaling_servers WHERE enabled = 1 ORDER BY created_at ASC'),
			insert: db.prepare(`
				INSERT INTO signaling_servers (id, name, url, enabled)
				VALUES (@id, @name, @url, @enabled)
			`),
			update: db.prepare('UPDATE signaling_servers SET name = @name, url = @url, enabled = @enabled WHERE id = @id'),
			delete: db.prepare('DELETE FROM signaling_servers WHERE id = ?')
		};
	}

	get(id: string): SignalingServerRow | null {
		return this.stmts.get.get(id) ?? null;
	}

	getAll(): SignalingServerRow[] {
		return this.stmts.getAll.all();
	}

	getEnabled(): SignalingServerRow[] {
		return this.stmts.getEnabled.all();
	}

	insert(row: InsertRow): void {
		this.stmts.insert.run(row);
	}

	update(id: string, name: string, url: string, enabled: number): void {
		this.stmts.update.run({ id, name, url, enabled });
	}

	delete(id: string): boolean {
		const result = this.stmts.delete.run(id);
		return result.changes > 0;
	}
}
