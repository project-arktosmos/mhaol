import { json } from '@sveltejs/kit';
import { basename } from 'node:path';
import type { RequestHandler } from './$types';
import type { LibraryItemRow } from 'database';

function mapItem(r: LibraryItemRow) {
	return {
		id: r.id,
		name: basename(r.path, '.' + r.extension),
		extension: r.extension,
		path: r.path,
		categoryId: r.category_id,
		createdAt: r.created_at
	};
}

export const GET: RequestHandler = async ({ locals }) => {
	const mediaTypes = locals.mediaTypeRepo.getAll().map((mt) => ({
		id: mt.id,
		label: mt.label
	}));

	const categories = locals.categoryRepo.getAll().map((c) => ({
		id: c.id,
		mediaTypeId: c.media_type_id,
		label: c.label
	}));

	type Item = ReturnType<typeof mapItem>;
	const itemsByCategory: Record<string, Item[]> = {};
	const itemsByType: Record<string, Item[]> = {};

	for (const category of categories) {
		const rows = locals.libraryItemRepo.getByCategory(category.id);
		if (category.id.endsWith('-uncategorized')) {
			const uncategorized = locals.libraryItemRepo.getUncategorizedByMediaType(
				category.mediaTypeId
			);
			itemsByCategory[category.id] = [...rows, ...uncategorized].map(mapItem);
		} else {
			itemsByCategory[category.id] = rows.map(mapItem);
		}
	}

	for (const mt of mediaTypes) {
		itemsByType[mt.id] = locals.libraryItemRepo.getByMediaType(mt.id).map(mapItem);
	}

	return json({ mediaTypes, categories, itemsByCategory, itemsByType });
};
