import { fetchJson } from 'ui-lib/transport/fetch-helpers';
import { gameListItemToDisplay, gameExtendedToDisplay } from 'addons/retroachievements/transform';
import { RA_CONSOLES } from 'addons/retroachievements/types';
import type { RaGameListItem, RaGameExtended } from 'addons/retroachievements/types';
import type { CatalogItem, CatalogFilterOption, CatalogTab } from 'ui-lib/types/catalog.type';
import type { CatalogKindStrategy } from 'ui-lib/services/catalog.service';

const ITEMS_PER_PAGE = 20;

function gameTag(title: string): string {
	if (title.startsWith('~')) {
		const end = title.indexOf('~', 1);
		if (end > 1) return title.substring(1, end);
	}
	return 'Originals';
}

function deriveCategoryTabs(games: RaGameListItem[]): CatalogTab[] {
	const counts = new Map<string, number>();
	for (const g of games) {
		const tag = gameTag(g.Title);
		counts.set(tag, (counts.get(tag) ?? 0) + 1);
	}
	const tabs: CatalogTab[] = [];
	if (counts.has('Originals')) tabs.push({ id: 'Originals', label: 'Originals' });
	for (const tag of [...counts.keys()].sort()) {
		if (tag !== 'Originals') tabs.push({ id: tag, label: tag });
	}
	return tabs;
}

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

function gameDetailToCatalogItem(detail: RaGameExtended): CatalogItem {
	const display = gameExtendedToDisplay(detail);
	return {
		id: String(display.id),
		kind: 'game' as const,
		title: display.title,
		sortTitle: display.title.toLowerCase(),
		year: display.released ?? null,
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
			consoleName: display.consoleName,
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
}

export const gameStrategy: CatalogKindStrategy = {
	kind: 'game',
	pinService: 'retroachievements',
	tabs: [{ id: 'Originals', label: 'Originals' }],
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
			totalPages,
			tabs: deriveCategoryTabs(games)
		};
	},

	async loadTab(tabId, page, filters) {
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
		const tabs = deriveCategoryTabs(games);
		const category =
			tabId && tabs.some((t) => t.id === tabId) ? tabId : (tabs[0]?.id ?? 'Originals');
		const filtered = games.filter((g) => gameTag(g.Title) === category);
		const totalPages = Math.ceil(filtered.length / ITEMS_PER_PAGE);
		const offset = (page - 1) * ITEMS_PER_PAGE;
		return {
			items: toGameCatalogItems(filtered.slice(offset, offset + ITEMS_PER_PAGE), consoleName),
			totalPages,
			tabs
		};
	},

	async resolveByIds(ids) {
		const results = await Promise.allSettled(
			ids.map((id) => fetchJson<RaGameExtended>(`/api/retroachievements/games/${id}`))
		);
		return results
			.filter(
				(r): r is PromiseFulfilledResult<RaGameExtended> =>
					r.status === 'fulfilled' && r.value != null
			)
			.map((r) => gameDetailToCatalogItem(r.value));
	}
};
