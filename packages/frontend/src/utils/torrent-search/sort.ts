import type { TorrentSearchResult, TorrentSearchSort } from '$types/torrent-search.type';

export function sortSearchResults(
	results: TorrentSearchResult[],
	sort: TorrentSearchSort
): TorrentSearchResult[] {
	const sorted = [...results];
	const multiplier = sort.direction === 'asc' ? 1 : -1;

	sorted.sort((a, b) => {
		switch (sort.field) {
			case 'seeders':
				return (a.seeders - b.seeders) * multiplier;
			case 'leechers':
				return (a.leechers - b.leechers) * multiplier;
			case 'size':
				return (a.size - b.size) * multiplier;
			case 'name':
				return a.name.localeCompare(b.name) * multiplier;
			case 'uploadedAt':
				return (a.uploadedAt.getTime() - b.uploadedAt.getTime()) * multiplier;
			default:
				return 0;
		}
	});

	return sorted;
}
