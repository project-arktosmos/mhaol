import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { getDatabase } from 'database';

interface TableInfo {
	name: string;
	columns: { name: string; type: string }[];
	rowCount: number;
}

export const GET: RequestHandler = async () => {
	const db = getDatabase();

	const tables = db
		.prepare(
			"SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
		)
		.all() as { name: string }[];

	const result: TableInfo[] = tables.map((table) => {
		const columns = db.prepare(`PRAGMA table_info('${table.name}')`).all() as {
			name: string;
			type: string;
		}[];

		const countRow = db.prepare(`SELECT COUNT(*) as count FROM "${table.name}"`).get() as {
			count: number;
		};

		return {
			name: table.name,
			columns: columns.map((c) => ({ name: c.name, type: c.type })),
			rowCount: countRow.count
		};
	});

	return json(result);
};
