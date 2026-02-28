import { json } from '@sveltejs/kit';
import { basename } from 'node:path';
import type { RequestHandler } from './$types';
import type { LibraryItemRow, LibraryItemLinkRow } from 'database';

interface MappedLink {
	serviceId: string;
	seasonNumber: number | null;
	episodeNumber: number | null;
}

function mapItem(r: LibraryItemRow, linkRows: LibraryItemLinkRow[], mediaTypeId: string) {
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
		mediaTypeId,
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

	// Auto-link completed YouTube downloads to library items
	const completedDownloads = locals.youtubeDownloadRepo.getByState('completed');
	for (const dl of completedDownloads) {
		if (!dl.output_path) continue;
		const itemId = locals.libraryItemRepo.existsByPath(dl.output_path);
		if (!itemId) continue;
		const existing = locals.libraryItemLinkRepo.getByItemAndService(itemId, 'youtube');
		if (!existing) {
			locals.libraryItemLinkRepo.upsert({
				id: crypto.randomUUID(),
				library_item_id: itemId,
				service: 'youtube',
				service_id: dl.video_id,
				season_number: null,
				episode_number: null
			});
		}
	}

	function mapRows(rows: LibraryItemRow[], mediaTypeId: string) {
		return rows.map((r) => {
			const linkRows = locals.libraryItemLinkRepo.getByItem(r.id);
			return mapItem(r, linkRows, mediaTypeId);
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
			itemsByCategory[category.id] = mapRows([...rows, ...uncategorized], category.mediaTypeId);
		} else {
			itemsByCategory[category.id] = mapRows(rows, category.mediaTypeId);
		}
	}

	for (const mt of mediaTypes) {
		itemsByType[mt.id] = mapRows(locals.libraryItemRepo.getByMediaType(mt.id), mt.id);
	}

	return json({ mediaTypes, categories, linkSources, itemsByCategory, itemsByType });
};
