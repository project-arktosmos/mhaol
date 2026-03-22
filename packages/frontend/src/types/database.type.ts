export interface DatabaseColumn {
	name: string;
	type: string;
}

export interface DatabaseTable {
	name: string;
	columns: DatabaseColumn[];
	rowCount: number;
}

export interface DatabasePagination {
	page: number;
	limit: number;
	total: number;
	totalPages: number;
}

export interface DatabaseTableDetail {
	table: string;
	columns: DatabaseColumn[];
	rows: Record<string, unknown>[];
	pagination: DatabasePagination;
}
