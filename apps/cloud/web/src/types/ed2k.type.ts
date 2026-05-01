import type { ID } from '$types/core.type';

export type Ed2kState =
	| 'initializing'
	| 'searching'
	| 'downloading'
	| 'seeding'
	| 'paused'
	| 'error';

export interface Ed2kFileInfo {
	id: number;
	name: string;
	fileHash: string;
	size: number;
	progress: number;
	downloadSpeed: number;
	uploadSpeed: number;
	peers: number;
	seeds: number;
	state: Ed2kState;
	addedAt: number;
	eta: number | null;
	outputPath: string | null;
	sourceUri: string;
}

export interface Ed2kStats {
	totalDownloaded: number;
	totalUploaded: number;
	downloadSpeed: number;
	uploadSpeed: number;
	activeFiles: number;
	serverConnected: boolean;
	serverName: string;
}

export interface Ed2kSearchResult {
	name: string;
	fileHash: string;
	size: number;
	sources: number;
	completeSources: number;
	ed2kLink: string;
	mediaType: string | null;
}

export interface Ed2kServer {
	name: string;
	host: string;
	port: number;
	userCount: number;
	fileCount: number;
	message: string;
	assignedId: number | null;
}

export interface AddEd2kRequest {
	source: string;
	downloadPath?: string;
	paused?: boolean;
}

export interface Ed2kServiceState {
	initialized: boolean;
	loading: boolean;
	error: string | null;
	files: Ed2kFileInfo[];
	stats: Ed2kStats | null;
	server: Ed2kServer | null;
	downloadPath: string;
	searchQuery: string;
	searching: boolean;
	searchResults: Ed2kSearchResult[];
}

export interface Ed2kSettings {
	id: ID;
	downloadPath: string;
}

export function ed2kStateLabel(state: Ed2kState): string {
	switch (state) {
		case 'initializing':
			return 'Initializing';
		case 'searching':
			return 'Searching';
		case 'downloading':
			return 'Downloading';
		case 'seeding':
			return 'Seeding';
		case 'paused':
			return 'Paused';
		case 'error':
			return 'Error';
		default:
			return state;
	}
}

export function ed2kStateColor(state: Ed2kState): string {
	switch (state) {
		case 'downloading':
			return 'primary';
		case 'seeding':
			return 'success';
		case 'paused':
			return 'warning';
		case 'error':
			return 'error';
		case 'searching':
		case 'initializing':
			return 'info';
		default:
			return 'neutral';
	}
}
