/** Raw response item from the apibay.org /q.php endpoint */
export interface PirateBayApiResult {
	id: string;
	name: string;
	info_hash: string;
	leechers: string;
	seeders: string;
	num_files: string;
	size: string;
	username: string;
	added: string;
	status: string;
	category: string;
	imdb: string;
}

/** Parsed, display-ready search result */
export interface TorrentSearchResult {
	id: string;
	name: string;
	infoHash: string;
	magnetLink: string;
	seeders: number;
	leechers: number;
	size: number;
	category: string;
	uploadedBy: string;
	uploadedAt: Date;
	isVip: boolean;
	isTrusted: boolean;
}

/** Sort options for search results */
export type TorrentSearchSortField = 'seeders' | 'leechers' | 'size' | 'name' | 'uploadedAt';
export type TorrentSearchSortDirection = 'asc' | 'desc';

export interface TorrentSearchSort {
	field: TorrentSearchSortField;
	direction: TorrentSearchSortDirection;
}

/** PirateBay top-level category codes */
export enum TorrentCategory {
	All = '0',
	Audio = '100',
	Video = '200',
	Applications = '300',
	Games = '400',
	Other = '600'
}

export const TORRENT_CATEGORY_LABELS: Record<TorrentCategory, string> = {
	[TorrentCategory.All]: 'All',
	[TorrentCategory.Audio]: 'Audio',
	[TorrentCategory.Video]: 'Video',
	[TorrentCategory.Applications]: 'Applications',
	[TorrentCategory.Games]: 'Games',
	[TorrentCategory.Other]: 'Other'
};
