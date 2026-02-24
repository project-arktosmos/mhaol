import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import crypto from 'node:crypto';

export const GET: RequestHandler = async ({ locals }) => {
	const rows = locals.libraryRepo.getAll();
	const libraries = rows.map((row) => ({
		id: row.id,
		name: row.name,
		path: row.path,
		mediaTypes: JSON.parse(row.media_types),
		dateAdded: row.date_added
	}));
	return json(libraries);
};

export const POST: RequestHandler = async ({ request, locals }) => {
	const body = await request.json();

	if (!body.name || !body.path || !Array.isArray(body.mediaTypes)) {
		return json({ error: 'Missing required fields: name, path, mediaTypes' }, { status: 400 });
	}

	const library = {
		id: crypto.randomUUID(),
		name: body.name,
		path: body.path,
		media_types: JSON.stringify(body.mediaTypes),
		date_added: Date.now()
	};

	locals.libraryRepo.insert(library);

	return json(
		{
			id: library.id,
			name: library.name,
			path: library.path,
			mediaTypes: body.mediaTypes,
			dateAdded: library.date_added
		},
		{ status: 201 }
	);
};
