import { fetchJson } from 'ui-lib/transport/fetch-helpers';
import { gameListItemToDisplay } from 'addons/retroachievements/transform';
import { RA_CONSOLES } from 'addons/retroachievements/types';
import type { RaGameListItem } from 'addons/retroachievements/types';
import type { CatalogItem, CatalogFilterOption } from 'ui-lib/types/catalog.type';
import type { CatalogKindStrategy } from 'ui-lib/services/catalog.service';

const ITEMS_PER_PAGE = 20;

function toGameCatalogItems(games: RaGameListItem[], consoleName: string): CatalogItem[] {
	return games.map((g) => {
		const display = gameListItemToDisplay(g);
		return {
			id: String(display.id),
			kind: 'game' as const,
			title: display.title,
			sortTitle: display.title.toLowerCase(),
			year: null,
			overview: null,
			posterUrl: display.imageIconUrl || null,
			backdropUrl: null,
			voteAverage: null,
			voteCount: null,
			parentId: null,
			position: null,
			source: 'retroachievements' as const,
			sourceId: String(display.id),
			createdAt: '',
			updatedAt: '',
			metadata: {
				retroachievementsId: display.id,
				consoleId: display.consoleId,
				consoleName,
				imageIconUrl: display.imageIconUrl,
				numAchievements: display.numAchievements,
				points: display.points,
				developer: display.developer ?? null,
				publisher: display.publisher ?? null,
				genre: display.genre ?? null,
				released: display.released ?? null,
				imageTitleUrl: display.imageTitleUrl ?? null,
				imageIngameUrl: display.imageIngameUrl ?? null,
				imageBoxArtUrl: display.imageBoxArtUrl ?? null,
				achievements: []
			}
		};
	});
}

const cachedGames = new Map<number, RaGameListItem[]>();

export const gameStrategy: CatalogKindStrategy = {
	kind: 'game',
	tabs: [{ id: 'browse', label: 'Browse' }],
	filterDefinitions: {
		console: {
			label: 'Console',
			loadOptions: async () => RA_CONSOLES.map((c) => ({ id: String(c.id), label: c.name }))
		}
	},

	async search(query, page, filters) {
		const consoleId = Number(filters.console) || RA_CONSOLES[0]?.id;
		if (!consoleId) return { items: [], totalPages: 1 };
		const consoleName = RA_CONSOLES.find((c) => c.id === consoleId)?.name ?? '';
		let games = cachedGames.get(consoleId);
		if (!games) {
			games =
				(await fetchJson<RaGameListItem[]>(`/api/retroachievements/games?console=${consoleId}`)) ??
				[];
			cachedGames.set(consoleId, games);
		}
		const q = query.toLowerCase();
		const filtered = games.filter((g) => g.Title.toLowerCase().includes(q));
		const totalPages = Math.ceil(filtered.length / ITEMS_PER_PAGE);
		const offset = (page - 1) * ITEMS_PER_PAGE;
		return {
			items: toGameCatalogItems(filtered.slice(offset, offset + ITEMS_PER_PAGE), consoleName),
			totalPages
		};
	},

	async loadTab(_tabId, page, filters) {
		const consoleId = Number(filters.console) || RA_CONSOLES[0]?.id;
		if (!consoleId) return { items: [], totalPages: 1 };
		const consoleName = RA_CONSOLES.find((c) => c.id === consoleId)?.name ?? '';
		let games = cachedGames.get(consoleId);
		if (!games) {
			games =
				(await fetchJson<RaGameListItem[]>(`/api/retroachievements/games?console=${consoleId}`)) ??
				[];
			cachedGames.set(consoleId, games);
		}
		const totalPages = Math.ceil(games.length / ITEMS_PER_PAGE);
		const offset = (page - 1) * ITEMS_PER_PAGE;
		return {
			items: toGameCatalogItems(games.slice(offset, offset + ITEMS_PER_PAGE), consoleName),
			totalPages
		};
	}
};
