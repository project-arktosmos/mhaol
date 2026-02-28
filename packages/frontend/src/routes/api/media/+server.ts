import { json } from '@sveltejs/kit';
import { basename } from 'node:path';
import type { RequestHandler } from './$types';
import type { LibraryItemRow, LibraryItemLinkRow } from 'database';

interface MappedLink {
	serviceId: string;
	seasonNumber: number | null;
	episodeNumber: number | null;
}

function mapItem(r: LibraryItemRow, linkRows: LibraryItemLinkRow[]) {
	const links: Record<string, MappedLink> = {};
	for (const link of linkRows) {
		links[link.service] = {
			serviceId: link.service_id,
			seasonNumber: link.season_number,
			episodeNumber: link.episode_number
		};
	}

	return {
		id: r.id,
		libraryId: r.library_id,
		name: basename(r.path, '.' + r.extension),
		extension: r.extension,
		path: r.path,
		categoryId: r.category_id,
		createdAt: r.created_at,
		links
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

	const linkSources = locals.linkSourceRepo.getAll().map((ls) => ({
		id: ls.id,
		service: ls.service,
		label: ls.label,
		mediaTypeId: ls.media_type_id,
		categoryId: ls.category_id
	}));

	function mapRows(rows: LibraryItemRow[]) {
		return rows.map((r) => {
			const linkRows = locals.libraryItemLinkRepo.getByItem(r.id);
			return mapItem(r, linkRows);
		});
	}

	type Item = ReturnType<typeof mapItem>;
	const itemsByCategory: Record<string, Item[]> = {};
	const itemsByType: Record<string, Item[]> = {};

	for (const category of categories) {
		const rows = locals.libraryItemRepo.getByCategory(category.id);
		if (category.id.endsWith('-uncategorized')) {
			const uncategorized = locals.libraryItemRepo.getUncategorizedByMediaType(
				category.mediaTypeId
			);
			itemsByCategory[category.id] = mapRows([...rows, ...uncategorized]);
		} else {
			itemsByCategory[category.id] = mapRows(rows);
		}
	}

	for (const mt of mediaTypes) {
		itemsByType[mt.id] = mapRows(locals.libraryItemRepo.getByMediaType(mt.id));
	}

	return json({ mediaTypes, categories, linkSources, itemsByCategory, itemsByType });
};
