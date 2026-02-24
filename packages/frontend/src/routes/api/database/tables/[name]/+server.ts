import { json, error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { getDatabase } from 'database';

export const GET: RequestHandler = async ({ params, url }) => {
	const db = getDatabase();
	const tableName = params.name;

	const tableExists = db
		.prepare(
			"SELECT 1 FROM sqlite_master WHERE type='table' AND name = ? AND name NOT LIKE 'sqlite_%'"
		)
		.get(tableName);

	if (!tableExists) {
		throw error(404, `Table "${tableName}" not found`);
	}

	const page = Math.max(1, Number(url.searchParams.get('page') ?? '1'));
	const limit = Math.min(100, Math.max(1, Number(url.searchParams.get('limit') ?? '20')));
	const offset = (page - 1) * limit;

	const countRow = db.prepare(`SELECT COUNT(*) as count FROM "${tableName}"`).get() as {
		count: number;
	};

	const rows = db.prepare(`SELECT * FROM "${tableName}" LIMIT ? OFFSET ?`).all(limit, offset);

	const columns = db.prepare(`PRAGMA table_info('${tableName}')`).all() as {
		name: string;
		type: string;
	}[];

	return json({
		table: tableName,
		columns: columns.map((c) => ({ name: c.name, type: c.type })),
		rows,
		pagination: {
			page,
			limit,
			total: countRow.count,
			totalPages: Math.ceil(countRow.count / limit)
		}
	});
};
